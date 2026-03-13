use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::SimConfig;
use crate::domain::Domain;
use crate::metrics::{MetricsLog, TickMetrics};
use crate::pacemaker;
use crate::plasticity;
use crate::propagation;
use crate::stimulus;
use crate::zone::ZoneManager;

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
    rng: ChaCha8Rng,
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
        let domain = Domain::from_config(
            &config.domain,
            &config.node_defaults,
            &config.edge_defaults,
        );
        let zone_manager = ZoneManager::from_config(&config.zones, &domain);
        let rng = ChaCha8Rng::seed_from_u64(config.simulation.seed);
        Simulation {
            domain,
            metrics: MetricsLog::new(),
            config,
            tick: 0,
            finished: false,
            rng,
            zone_manager,
        }
    }

    /// Nombre total de ticks prévu.
    pub fn total_ticks(&self) -> usize {
        self.config.simulation.total_ticks
    }

    /// Exécuter **un seul tick** de la simulation.
    /// V2 : séquence en 6 phases — mesure → régulation → stimulus → pacemaker → propagation → dissipation → plasticité
    pub fn step(&mut self) -> TickMetrics {
        let tick = self.tick;
        let config = &self.config;

        // === Phase 0 : Mesure de l'activité des zones ===
        if config.zones.enabled {
            self.zone_manager.measure(&self.domain);
        }

        // === Phase 1 : Régulation PID ===
        if config.zones.enabled {
            let zones_config = config.zones.clone();
            self.zone_manager.regulate(&mut self.domain, &zones_config);
        }

        // === Phase 2 : Injection de stimuli externes ===
        stimulus::inject_stimuli(&mut self.domain, &config.stimuli, tick);

        // === Phase 2b : Pacemakers ===
        if !config.pacemakers.is_empty() {
            pacemaker::apply_pacemakers(&mut self.domain, &config.pacemakers, tick);
        }

        // === Phase 3 : Propagation ===
        let influences = propagation::compute_influences(&self.domain, &config.propagation);
        let _newly_activated = propagation::apply_influences(&mut self.domain, &influences);

        // === Phase 4 : Dissipation (avec jitter aléatoire sur le decay) ===
        let base_decay = config.dissipation.activation_decay;
        let jitter = config.dissipation.decay_jitter;
        let activation_min = config.dissipation.activation_min;
        for node in self.domain.nodes.iter_mut() {
            node.update_fatigue(config.dissipation.fatigue_gain, config.dissipation.fatigue_recovery);
            node.update_inhibition(config.dissipation.inhibition_gain, config.dissipation.inhibition_decay);
            node.update_trace(config.dissipation.trace_gain, config.dissipation.trace_decay);
            node.update_excitability_from_trace();
            // Decay avec jitter : taux ± jitter% par nœud par tick
            let effective_decay = if jitter > 0.0 {
                let j: f64 = self.rng.gen_range(-jitter..jitter);
                (base_decay * (1.0 + j)).clamp(0.0, 1.0)
            } else {
                base_decay
            };
            node.decay_activation(effective_decay, activation_min);
        }

        // === Phase 5 : Plasticité + consolidation ===
        plasticity::update_plasticity(&mut self.domain, &config.plasticity, &config.consolidation);

        // === Phase 6 : Métriques ===
        self.metrics.record(tick, &self.domain, &self.zone_manager);
        let tick_metrics = self.metrics.snapshots.last().unwrap().clone();

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
    let mut sim = Simulation::new(config.clone());

    observer.on_init(&sim.domain, config);

    let ticks_label = if sim.total_ticks() == 0 { "∞".to_string() } else { sim.total_ticks().to_string() };
    println!(
        "=== Simulation V2 ===\nTopologie: {} | Nœuds: {}, Liaisons: {}, Ticks: {} | Zones: {}",
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
            println!(
                "  tick {:>5} | actifs: {:>5} | énergie: {:.3}",
                tick, tick_metrics.active_nodes, tick_metrics.global_energy
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
