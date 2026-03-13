use rayon::prelude::*;

use crate::config::SimConfig;
use crate::domain::Domain;
use crate::dopamine::DopamineSystem;
use crate::input::InputEncoder;
use crate::metrics::{MetricsLog, TickMetrics};
use crate::output::OutputReader;
use crate::pacemaker;
use crate::perf::{PerfMonitor, Phase};
use crate::propagation;
use crate::reward::RewardSystem;
use crate::stdp;
use crate::stimulus;
use crate::zone::ZoneManager;

/// Initialiser le pool de threads rayon en détectant automatiquement le nombre de cœurs.
pub fn init_thread_pool() {
    let n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build_global()
        .ok();
    eprintln!("[Stretch] Thread pool rayon : {} threads (détecté)", rayon::current_num_threads());
}

// ---------------------------------------------------------------------------
// Observer trait
// ---------------------------------------------------------------------------

pub trait SimulationObserver {
    fn on_init(&mut self, _domain: &Domain, _config: &SimConfig) {}
    fn on_tick(&mut self, _tick: usize, _domain: &Domain, _metrics: &TickMetrics) -> bool {
        true
    }
    fn on_finish(&mut self, _domain: &Domain, _metrics: &MetricsLog) {}
}

pub struct NullObserver;
impl SimulationObserver for NullObserver {}

// ---------------------------------------------------------------------------
// V4 Training trial
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Trial {
    pub class: usize,
    pub start_tick: usize,
    pub presentation_ticks: usize,
    pub read_delay: usize,
}

// ---------------------------------------------------------------------------
// Simulation V4 — optimized with kernel cache, reusable buffers, PerfMonitor
// ---------------------------------------------------------------------------

pub struct Simulation {
    pub domain: Domain,
    pub metrics: MetricsLog,
    pub config: SimConfig,
    pub tick: usize,
    pub finished: bool,
    pub zone_manager: ZoneManager,
    pub dopamine_system: DopamineSystem,
    pub reward_system: RewardSystem,
    pub input_encoder: Option<InputEncoder>,
    pub output_reader: Option<OutputReader>,
    pub trials: Vec<Trial>,
    pub current_trial: usize,
    pub last_decision: Option<usize>,
    pub last_target: Option<usize>,
    pub correct_count: usize,
    pub total_evaluated: usize,
    pub reward_center: Option<[f64; 3]>,
    // --- Performance: reusable buffers (avoid allocation per tick) ---
    buf_source_contribs: Vec<f64>,
    buf_influences: Vec<f64>,
    buf_activation_ticks: Vec<Option<usize>>,
    buf_node_delta_dopa: Vec<f64>,
    cached_dopa_center: Option<[f64; 3]>,
    // --- Monitoring ---
    pub perf: PerfMonitor,
}

pub struct SimulationResult {
    pub metrics: MetricsLog,
    pub domain: Domain,
}

impl Simulation {
    pub fn new(config: SimConfig) -> Self {
        init_thread_pool();

        let mut domain = Domain::from_config(
            &config.domain,
            &config.node_defaults,
            &config.edge_defaults,
        );

        domain.assign_neuron_types(
            config.neuron_types.inhibitory_fraction,
            config.simulation.seed,
        );

        let zone_manager = ZoneManager::from_config(&config.zones, &domain);
        let dopamine_system = DopamineSystem::new(&config.dopamine);
        let reward_system = RewardSystem::new();

        let input_encoder = Some(InputEncoder::new(&config.input, &domain));

        let output_reader = Some(OutputReader::new(&config.output, &domain));

        // Pre-compute kernel weights into CSR (eliminates ~577k exp() per tick)
        domain.set_incoming_kernel_weights(&config.propagation);

        let n_nodes = domain.num_nodes();
        let perf = PerfMonitor::new(100); // report every 100 ticks

        Simulation {
            domain,
            metrics: MetricsLog::new(),
            config,
            tick: 0,
            finished: false,
            zone_manager,
            dopamine_system,
            reward_system,
            input_encoder,
            output_reader,
            trials: Vec::new(),
            current_trial: 0,
            last_decision: None,
            last_target: None,
            correct_count: 0,
            total_evaluated: 0,
            reward_center: None,
            buf_source_contribs: vec![0.0; n_nodes],
            buf_influences: vec![0.0; n_nodes],
            buf_activation_ticks: vec![None; n_nodes],
            buf_node_delta_dopa: Vec::new(),
            cached_dopa_center: None,
            perf,
        }
    }

    pub fn schedule_trials(&mut self, trials: Vec<Trial>) {
        self.trials = trials;
        self.current_trial = 0;
    }

    /// Configuration complète V4 : sélection spatiale I/O + scheduling des trials.
    pub fn setup_v4_training(&mut self) -> usize {
        let config = self.config.clone();
        let extent = config.domain.domain_extent;
        let group_size = config.input.group_size;
        let mid = extent / 2.0;
        let mut used_nodes: Vec<usize> = Vec::new();

        let in0 = self.domain.select_nearest_nodes([0.0, mid, mid], group_size, &used_nodes);
        used_nodes.extend(&in0);
        let in1 = self.domain.select_nearest_nodes([extent, mid, mid], group_size, &used_nodes);
        used_nodes.extend(&in1);
        let out0 = self.domain.select_nearest_nodes([extent * 0.25, mid, mid], group_size, &used_nodes);
        used_nodes.extend(&out0);
        let out1 = self.domain.select_nearest_nodes([extent * 0.75, mid, mid], group_size, &used_nodes);

        if let Some(ref mut encoder) = self.input_encoder {
            encoder.groups = vec![in0, in1];
        }
        if let Some(ref mut reader) = self.output_reader {
            reader.groups = vec![out0, out1];
        }

        let presentation_ticks = 5;
        let read_delay = config.output.read_delay;
        let inter_trial_gap = 15;
        let trial_period = presentation_ticks + read_delay + inter_trial_gap + 1;
        let num_classes = config.input.num_classes;
        let total_ticks = config.simulation.total_ticks;
        let max_trials = if total_ticks > 0 { total_ticks / trial_period } else { 200 };
        let warmup_ticks = 20;

        let mut trials = Vec::with_capacity(max_trials);
        for i in 0..max_trials {
            let class = i % num_classes;
            let start_tick = warmup_ticks + i * trial_period;
            if total_ticks > 0 && start_tick + presentation_ticks + read_delay >= total_ticks {
                break;
            }
            trials.push(Trial {
                class,
                start_tick,
                presentation_ticks,
                read_delay,
            });
        }

        let n_trials = trials.len();
        self.schedule_trials(trials);
        n_trials
    }

    pub fn total_ticks(&self) -> usize {
        self.config.simulation.total_ticks
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_evaluated == 0 { 0.0 } else { self.correct_count as f64 / self.total_evaluated as f64 }
    }

    /// V4 : Exécuter un seul tick (optimized).
    pub fn step(&mut self) -> TickMetrics {
        let tick = self.tick;
        let config = &self.config;

        self.perf.begin_tick();

        // === Phase 0+1 : Zones + PID ===
        self.zone_manager.measure(&self.domain);
        self.zone_manager.regulate(&mut self.domain, &config.zones);

        self.perf.end_phase(Phase::Zones);

        // === Phase 2 : Stimuli + pacemakers ===
        // Stimuli classiques ignorés si InputEncoder actif (V4)
        if self.input_encoder.as_ref().map_or(true, |e| e.groups.is_empty()) {
            stimulus::inject_stimuli(&mut self.domain, &config.stimuli, tick);
        }
        if !config.pacemakers.is_empty() {
            pacemaker::apply_pacemakers(&mut self.domain, &config.pacemakers, tick);
        }

        // === Phase 3 : V4 — injection d'entrée ===
        if let Some(ref encoder) = self.input_encoder {
            if self.current_trial < self.trials.len() {
                let trial = &self.trials[self.current_trial];
                if tick == trial.start_tick {
                    for node in self.domain.nodes.iter_mut() {
                        node.activation = 0.0;
                    }
                }
                if tick >= trial.start_tick && tick < trial.start_tick + trial.presentation_ticks {
                    encoder.inject(&mut self.domain, trial.class, config.input.intensity);
                    self.last_target = Some(trial.class);
                }
            }
        }

        self.perf.end_phase(Phase::StimInput);

        // === Phase 4 : Propagation (cached kernels, reused buffers) ===
        propagation::compute_source_contribs(
            &self.domain,
            &config.propagation,
            &self.zone_manager.gain_mods,
            &mut self.buf_source_contribs,
        );
        propagation::compute_influences_csr(
            &self.domain,
            &self.buf_source_contribs,
            &mut self.buf_influences,
        );
        propagation::apply_influences(&mut self.domain, &self.buf_influences);

        self.perf.end_phase(Phase::Propagation);

        // === Phase 5 : Dissipation parallèle ===
        {
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

            self.domain
                .nodes
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, node)| {
                    let effective_decay = if jitter > 0.0 {
                        let h = (i as u64)
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(jitter_seed)
                            .wrapping_mul(1442695040888963407);
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
        }

        self.perf.end_phase(Phase::Dissipation);

        // === Phase 5b : Décroissance naturelle dopamine ===
        self.dopamine_system.update(0.0, &config.dopamine);

        // === Phase 6 : Plasticité + STDP + éligibilité + dopamine (with reusable buffers) ===
        {
            let dopamine_level = self.dopamine_system.level;
            let reward_center = self.reward_center;
            stdp::update_plasticity_stdp_budget(
                &mut self.domain,
                &config.consolidation,
                &config.stdp,
                &config.synaptic_budget,
                &config.edge_defaults,
                &config.eligibility,
                &config.dopamine,
                dopamine_level,
                reward_center,
                tick,
                &mut self.buf_activation_ticks,
                &mut self.buf_node_delta_dopa,
                &mut self.cached_dopa_center,
            );
        }

        // Sync conductance cache after plasticity modified edges
        self.domain.sync_conductances();

        self.perf.end_phase(Phase::Plasticity);

        // === Phase 7 : V4 — readout de sortie ===
        if let Some(ref reader) = self.output_reader {
            if self.current_trial < self.trials.len() {
                let trial = &self.trials[self.current_trial];
                let read_tick = trial.start_tick + trial.presentation_ticks + trial.read_delay;
                if tick == read_tick {
                    let result = reader.readout(&self.domain);
                    self.last_decision = Some(result.decision);

                    if self.total_evaluated > 0 && self.total_evaluated % 10 == 0 {
                        eprintln!("  [DIAG t={} trial={}] target={} dec={} scores=[{:.1}, {:.1}] margin={:.1}",
                            tick, self.current_trial, self.last_target.unwrap_or(99), result.decision,
                            result.scores[0], result.scores[1], result.margin);
                    }

                    if let Some(target) = self.last_target {
                        self.total_evaluated += 1;
                        let correct = result.decision == target;
                        if correct {
                            self.correct_count += 1;
                        }

                        let reward_val = if correct {
                            config.reward.reward_positive
                        } else {
                            config.reward.reward_negative
                        };
                        self.reward_system.set_reward(reward_val);
                        self.dopamine_system.update(reward_val, &config.dopamine);

                        if config.dopamine.spatial_lambda > 0.0 && config.dopamine.spatial_lambda > 0.0 {
                            let focus_group_idx = if correct { target } else { result.decision };
                            if let Some(ref reader_ref) = self.output_reader {
                                let focus_group = &reader_ref.groups[focus_group_idx];
                                let n = focus_group.len() as f64;
                                if n > 0.0 {
                                    let (mut cx, mut cy, mut cz) = (0.0_f64, 0.0_f64, 0.0_f64);
                                    for &idx in focus_group {
                                        let p = self.domain.positions[idx];
                                        cx += p[0]; cy += p[1]; cz += p[2];
                                    }
                                    self.reward_center = Some([cx/n, cy/n, cz/n]);
                                }
                            }
                        } else {
                            self.reward_center = None;
                        }
                    }

                    self.current_trial += 1;
                }
            }
        }

        self.reward_system.clear();

        self.perf.end_phase(Phase::ReadoutReward);

        // === Phase 9 : Métriques ===
        let snap_interval = config.simulation.snapshot_interval.max(1);
        if tick % snap_interval == 0 {
            self.metrics.record(
                tick,
                &self.domain,
                &self.zone_manager,
                self.reward_system.cumulative,
                self.dopamine_system.level,
                self.last_decision,
                self.accuracy(),
            );
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
            current_reward: 0.0,
            cumulative_reward: 0.0,
            dopamine_level: 0.0,
            mean_eligibility: 0.0,
            output_decision: None,
            accuracy: 0.0,
        });

        self.perf.end_phase(Phase::Metrics);
        self.perf.end_tick(tick);

        self.tick += 1;
        if config.simulation.total_ticks > 0 && self.tick >= config.simulation.total_ticks {
            self.finished = true;
        }

        tick_metrics
    }

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

pub fn run_with_observer(config: &SimConfig, observer: &mut dyn SimulationObserver) -> SimulationResult {
    let t_start = std::time::Instant::now();
    let mut sim = Simulation::new(config.clone());
    let t_init = std::time::Instant::now();

    observer.on_init(&sim.domain, config);

    let ticks_label = if sim.total_ticks() == 0 { "∞".to_string() } else { sim.total_ticks().to_string() };
    let v4_label = " [dopamine+reward]";
    eprintln!(
        "[Stretch] Init: {:.1}ms | Nœuds: {}, Liaisons: {}, Zones: {}",
        (t_init - t_start).as_secs_f64() * 1000.0,
        sim.domain.num_nodes(),
        sim.domain.num_edges(),
        sim.zone_manager.num_zones()
    );
    println!(
        "=== Simulation V4{} ===\nTopologie: {} | Nœuds: {}, Liaisons: {}, Ticks: {} | Zones: {}",
        v4_label,
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
            let acc_str = if sim.total_evaluated > 0 {
                format!(" | accuracy: {:.1}% ({}/{})", sim.accuracy() * 100.0, sim.correct_count, sim.total_evaluated)
            } else {
                String::new()
            };
            eprintln!(
                "  tick {:>5} | actifs: {:>5} | énergie: {:.3} | dopa: {:.3}{} | {:.1}ms/tick",
                tick, tick_metrics.active_nodes, tick_metrics.global_energy,
                sim.dopamine_system.level,
                acc_str,
                t_init.elapsed().as_secs_f64() * 1000.0 / (tick + 1) as f64
            );
        }
    }

    let active_final = sim.domain.nodes.iter().filter(|n| n.is_active()).count();
    let energy_final: f64 = sim.domain.nodes.iter().map(|n| n.activation).sum();
    let max_trace = sim.domain.nodes.iter().map(|n| n.memory_trace).fold(0.0_f64, f64::max);
    let max_cond = sim.domain.edges.iter().map(|e| e.conductance).fold(0.0_f64, f64::max);

    println!("\n=== Fin simulation V4 ===");
    println!("  Nœuds actifs finaux : {}", active_final);
    println!("  Énergie finale      : {:.4}", energy_final);
    println!("  Trace max           : {:.4}", max_trace);
    println!("  Conductance max     : {:.4}", max_cond);
    if sim.total_evaluated > 0 {
        println!("  Accuracy finale     : {:.1}% ({}/{})", sim.accuracy() * 100.0, sim.correct_count, sim.total_evaluated);
    }
    println!("  Dopamine finale     : {:.4}", sim.dopamine_system.level);
    println!("  Récompense cumulée  : {:.4}", sim.reward_system.cumulative);

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

pub fn run(config: &SimConfig) -> SimulationResult {
    run_with_observer(config, &mut NullObserver)
}
