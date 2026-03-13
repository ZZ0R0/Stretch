use rayon::prelude::*;

use crate::config::SimConfig;
use crate::domain::Domain;
use crate::metrics::{MetricsLog, TickMetrics};
use crate::pacemaker;
use crate::propagation;
use crate::stdp;
use crate::stimulus;
use crate::zone::ZoneManager;

/// Initialiser le pool de threads rayon en détectant automatiquement le nombre de cœurs.
/// Affiche le nombre de threads utilisés.
pub fn init_thread_pool() {
    let n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build_global()
        .ok(); // Ignore si déjà initialisé
    eprintln!("[Stretch] Thread pool rayon : {} threads (détecté)", rayon::current_num_threads());
}

// ---------------------------------------------------------------------------
// Observer trait — hooks / callbacks pour connecter la visualisation
// ---------------------------------------------------------------------------

/// Trait d'observation : implémenté par la visualisation ou tout autre connecteur.
/// Toutes les méthodes ont une implémentation par défaut vide (no-op).
pub trait SimulationObserver {
    /// Appelé une fois au démarrage, avant le premier tick.
    fn on_init(&mut self, _domain: &Domain, _config: &SimConfig) {}

    /// Appelé **après** chaque tick.
    /// Retourne `false` pour demander l'arrêt anticipé de la simulation.
    fn on_tick(&mut self, _tick: usize, _domain: &Domain, _metrics: &TickMetrics) -> bool {
        true
    }

    /// Appelé une fois à la fin de la simulation.
    fn on_finish(&mut self, _domain: &Domain, _metrics: &MetricsLog) {}
}

/// Observeur vide (pour le mode headless / CLI).
pub struct NullObserver;
impl SimulationObserver for NullObserver {}

// ---------------------------------------------------------------------------
// Simulation — moteur pas-à-pas
// ---------------------------------------------------------------------------

/// État complet d'une simulation en cours.
pub struct Simulation {
    pub domain: Domain,
    pub metrics: MetricsLog,
    pub config: SimConfig,
    pub tick: usize,
    pub finished: bool,
    /// V2 : gestionnaire de zones
    pub zone_manager: ZoneManager,
}

/// Résultat complet d'une simulation terminée.
pub struct SimulationResult {
    pub metrics: MetricsLog,
    pub domain: Domain,
}

impl Simulation {
    /// Créer une nouvelle simulation à partir d'une configuration.
    pub fn new(config: SimConfig) -> Self {
        // S'assurer que le thread pool est initialisé
        init_thread_pool();

        let mut domain = Domain::from_config(
            &config.domain,
            &config.node_defaults,
            &config.edge_defaults,
        );

        // V3 : assigner les types neuronaux (E/I)
        if config.neuron_types.enabled {
            domain.assign_neuron_types(
                config.neuron_types.inhibitory_fraction,
                config.simulation.seed,
            );
        }

        let zone_manager = ZoneManager::from_config(&config.zones, &domain);
        Simulation {
            domain,
            metrics: MetricsLog::new(),
            config,
            tick: 0,
            finished: false,
            zone_manager,
        }
    }

    /// Nombre total de ticks prévu.
    pub fn total_ticks(&self) -> usize {
        self.config.simulation.total_ticks
    }

    /// Exécuter **un seul tick** de la simulation.
    /// V3 : séquence en 8 phases — mesure → régulation → stimulus → pacemaker → propagation → dissipation → plasticité+STDP → normalisation
    pub fn step(&mut self) -> TickMetrics {
        let tick = self.tick;
        let config = &self.config;

        // Profiling : mesurer chaque phase (affichage aux ticks 0, 50, 100)
        let do_profile = tick < 3 || tick == 50 || tick == 100;
        let t0 = std::time::Instant::now();

        // === Phase 0 : Mesure de l'activité des zones ===
        if config.zones.enabled {
            self.zone_manager.measure(&self.domain);
        }

        // === Phase 1 : Régulation PID (direct ou indirect V3) ===
        if config.zones.enabled {
            self.zone_manager.regulate(&mut self.domain, &config.zones);
        }

        let t1 = std::time::Instant::now();

        // === Phase 2 : Injection de stimuli externes ===
        stimulus::inject_stimuli(&mut self.domain, &config.stimuli, tick);

        // === Phase 2b : Pacemakers ===
        if !config.pacemakers.is_empty() {
            pacemaker::apply_pacemakers(&mut self.domain, &config.pacemakers, tick);
        }

        let t2 = std::time::Instant::now();

        // === Phase 3 : Propagation signée V3 (E/I + gain_mods zone) ===
        let influences = propagation::compute_influences(
            &self.domain,
            &config.propagation,
            &self.zone_manager.gain_mods,
        );
        let _newly_activated = propagation::apply_influences(&mut self.domain, &influences);

        let t3 = std::time::Instant::now();

        // === Phase 4 : Dissipation parallèle ===
        let base_decay = config.dissipation.activation_decay;
        let jitter = config.dissipation.decay_jitter;
        let activation_min = config.dissipation.activation_min;
        let fatigue_gain = config.dissipation.fatigue_gain;
        let fatigue_recovery = config.dissipation.fatigue_recovery;
        let inhibition_gain = config.dissipation.inhibition_gain;
        let inhibition_decay_rate = config.dissipation.inhibition_decay;
        let trace_gain = config.dissipation.trace_gain;
        let trace_decay = config.dissipation.trace_decay;
        let jitter_seed = tick as u64;

        // Dissipation parallèle sur tous les nœuds (jitter calculé inline via hash rapide)
        self.domain
            .nodes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, node)| {
                let effective_decay = if jitter > 0.0 {
                    // Hash rapide pour générer un jitter déterministe par (tick, node)
                    let h = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(jitter_seed).wrapping_mul(1442695040888963407);
                    let j = ((h >> 33) as f64 / (1u64 << 31) as f64 - 1.0) * jitter;
                    (base_decay * (1.0 + j)).clamp(0.0, 1.0)
                } else {
                    base_decay
                };
                node.update_fatigue(fatigue_gain, fatigue_recovery);
                node.update_inhibition(inhibition_gain, inhibition_decay_rate);
                node.update_trace(trace_gain, trace_decay);
                node.update_excitability_from_trace();
                node.decay_activation(effective_decay, activation_min);
            });

        let t4 = std::time::Instant::now();

        // === Phase 5+6+7 : Plasticité + STDP + budget fusionnés (un seul passage sur les arêtes) ===
        stdp::update_plasticity_stdp_budget(
            &mut self.domain,
            &config.plasticity,
            &config.consolidation,
            &config.stdp,
            &config.synaptic_budget,
            &config.edge_defaults,
            tick,
        );

        let t5 = std::time::Instant::now();

        // === Phase 8 : Métriques (complètes uniquement aux snapshot_interval) ===
        let snap_interval = config.simulation.snapshot_interval.max(1);
        if tick % snap_interval == 0 {
            self.metrics.record(tick, &self.domain, &self.zone_manager);
        }
        let tick_metrics = self.metrics.snapshots.last().cloned().unwrap_or(TickMetrics {
            tick,
            active_nodes: 0,
            global_energy: 0.0,
            max_activation: 0.0,
            mean_conductance: 0.0,
            max_conductance: 0.0,
            mean_memory_trace: 0.0,
            max_memory_trace: 0.0,
            mean_fatigue: 0.0,
            consolidated_edges: 0,
            num_zones: 0,
            mean_pid_error: 0.0,
            mean_pid_output: 0.0,
            zone_activity_mean: 0.0,
            active_excitatory: 0,
            active_inhibitory: 0,
            excitatory_energy: 0.0,
            inhibitory_energy: 0.0,
        });

        let t8 = std::time::Instant::now();

        if do_profile {
            eprintln!(
                "[PROFILE tick {}] zones:{:.1}ms stim:{:.1}ms PROPAG:{:.1}ms dissip:{:.1}ms PLAST+STDP+BUD:{:.1}ms metrics:{:.1}ms | TOTAL:{:.1}ms",
                tick,
                (t1 - t0).as_secs_f64() * 1000.0,
                (t2 - t1).as_secs_f64() * 1000.0,
                (t3 - t2).as_secs_f64() * 1000.0,
                (t4 - t3).as_secs_f64() * 1000.0,
                (t5 - t4).as_secs_f64() * 1000.0,
                (t8 - t5).as_secs_f64() * 1000.0,
                (t8 - t0).as_secs_f64() * 1000.0,
            );
        }

        self.tick += 1;
        if config.simulation.total_ticks > 0 && self.tick >= config.simulation.total_ticks {
            self.finished = true;
        }

        tick_metrics
    }

    /// Consommer la simulation et retourner le résultat final.
    pub fn into_result(self) -> SimulationResult {
        SimulationResult {
            metrics: self.metrics,
            domain: self.domain,
        }
    }
}

// ---------------------------------------------------------------------------
// Fonction de commodité : exécution complète avec observer
// ---------------------------------------------------------------------------

/// Lancer la simulation complète avec un observer.
pub fn run_with_observer(config: &SimConfig, observer: &mut dyn SimulationObserver) -> SimulationResult {
    let t_start = std::time::Instant::now();
    let mut sim = Simulation::new(config.clone());
    let t_init = std::time::Instant::now();

    observer.on_init(&sim.domain, config);

    let ticks_label = if sim.total_ticks() == 0 { "∞".to_string() } else { sim.total_ticks().to_string() };
    eprintln!(
        "[Stretch] Init: {:.1}ms | Nœuds: {}, Liaisons: {}, Zones: {}",
        (t_init - t_start).as_secs_f64() * 1000.0,
        sim.domain.num_nodes(),
        sim.domain.num_edges(),
        sim.zone_manager.num_zones()
    );
    println!(
        "=== Simulation V3 ===\nTopologie: {} | Nœuds: {}, Liaisons: {}, Ticks: {} | Zones: {}",
        config.domain.topology,
        sim.domain.num_nodes(),
        sim.domain.num_edges(),
        ticks_label,
        sim.zone_manager.num_zones()
    );

    while !sim.finished {
        let tick = sim.tick;
        let tick_metrics = sim.step();

        let keep_going = observer.on_tick(tick, &sim.domain, &tick_metrics);
        if !keep_going {
            break;
        }

        if tick % 100 == 0 {
            eprintln!(
                "  tick {:>5} | actifs: {:>5} | énergie: {:.3} | {:.1}ms/tick",
                tick, tick_metrics.active_nodes, tick_metrics.global_energy,
                t_init.elapsed().as_secs_f64() * 1000.0 / (tick + 1) as f64
            );
        }
    }

    let active_final = sim.domain.nodes.iter().filter(|n| n.is_active()).count();
    let energy_final: f64 = sim.domain.nodes.iter().map(|n| n.activation).sum();
    let max_trace = sim.domain.nodes.iter().map(|n| n.memory_trace).fold(0.0_f64, f64::max);
    let max_cond = sim.domain.edges.iter().map(|e| e.conductance).fold(0.0_f64, f64::max);

    println!("\n=== Fin simulation ===");
    println!("  Nœuds actifs finaux : {}", active_final);
    println!("  Énergie finale      : {:.4}", energy_final);
    println!("  Trace max           : {:.4}", max_trace);
    println!("  Conductance max     : {:.4}", max_cond);

    let top = MetricsLog::top_edges(&sim.domain, 10);
    if !top.is_empty() {
        println!("\n  Top-10 liaisons les plus utilisées :");
        for (from, to, count, cond) in &top {
            println!("    {} -> {} : usage={}, conductance={:.4}", from, to, count, cond);
        }
    }

    observer.on_finish(&sim.domain, &sim.metrics);

    sim.into_result()
}

/// Lancer la simulation complète sans observer (mode headless).
pub fn run(config: &SimConfig) -> SimulationResult {
    run_with_observer(config, &mut NullObserver)
}
