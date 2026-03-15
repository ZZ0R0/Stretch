use rayon::prelude::*;

use std::sync::Arc;

use crate::calibration;
use crate::config::{SimConfig, V5TaskMode, V5BaselineMode};
use crate::diagnostics::SustainTracker;
use crate::domain::Domain;
use crate::dopamine::DopamineSystem;
use crate::gpu::{GpuContext, GpuParams};
use crate::input::InputEncoder;
use crate::metrics::{MetricsLog, TickMetrics};
use crate::output::OutputReader;
use crate::pacemaker;
use crate::perf::{PerfMonitor, Phase};
use crate::propagation;
use crate::reward::RewardSystem;
use crate::stdp;
use crate::stimulus;
use crate::sustained;
use crate::task;
use crate::zone::ZoneManager;

/// Compute backend: CPU (default) or GPU (wgpu).
pub enum ComputeBackend {
    Cpu,
    Gpu(GpuContext),
}

// ---------------------------------------------------------------------------
// VizSnapshot – lightweight render-only data for the visualization
// ---------------------------------------------------------------------------

/// Aggregated metrics for the sidebar (no per-node iteration needed).
#[derive(Clone)]
pub struct VizMetrics {
    pub active_count: usize,
    pub total_energy: f32,
    pub max_activation: f32,
    pub mean_conductance: f32,
    pub max_conductance: f32,
    pub mean_trace: f32,
    pub max_trace: f32,
    pub mean_fatigue: f32,
    pub dopamine_level: f32,
    pub current_trial: usize,
    pub total_trials: usize,
    pub correct_count: usize,
    pub total_evaluated: usize,
    pub accuracy: f32,
    pub last_decision: Option<usize>,
    pub last_target: Option<usize>,
    pub consolidated_edges: usize,
    pub excitatory_energy: f32,
    pub inhibitory_energy: f32,
    pub active_excitatory: usize,
    pub active_inhibitory: usize,
    pub num_zones: usize,
    pub zone_activity_mean: f64,
    pub mean_pid_error: f64,
    pub mean_pid_output: f64,
    pub pid_mode: String,
    // V6
    pub v6_sparsity_enabled: bool,
    pub v6_dopa_mod_enabled: bool,
    pub v6_novelty_active: usize,
}

/// Lightweight snapshot for visualization — contains only rendering data.
#[derive(Clone)]
pub struct VizSnapshot {
    pub tick: usize,
    pub finished: bool,
    /// 3D positions (constant after init, shared via Arc).
    pub positions: Arc<Vec<[f64; 3]>>,
    /// Per-node activation values (updated from GPU/CPU state).
    pub activations: Vec<f32>,
    /// Per-node memory trace (for trace view mode).
    pub memory_traces: Vec<f32>,
    /// Per-node fatigue (for fatigue view mode).
    pub fatigues: Vec<f32>,
    /// Per-node neuron type: true = excitatory.
    pub is_excitatory: Arc<Vec<bool>>,
    /// Indices of active nodes (activation > threshold).
    pub active_indices: Vec<usize>,
    /// Aggregated sidebar metrics.
    pub metrics: VizMetrics,
    /// Sparkline data (last N snapshots' global_energy).
    pub energy_history: Vec<f32>,
}

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
// Abstraction traits (E3) — enable future interchangeable backends / rules
// ---------------------------------------------------------------------------

/// Compute backend abstraction.
pub trait ComputeEngine {
    fn compute_influences(&self, domain: &Domain, source_contribs: &[f64], out: &mut [f64]);
    fn update_plasticity(&self, domain: &mut Domain, tick: usize, config: &SimConfig);
    fn normalize_budget(&self, domain: &mut Domain, budget: f64);
}

/// Plasticity rule abstraction (STDP, Hebbian, or future).
pub trait PlasticityRule {
    fn update(&self, domain: &mut Domain, tick: usize, config: &SimConfig);
}

/// Experimental protocol abstraction.
pub trait ExperimentProtocol {
    fn next_trial(&mut self, tick: usize) -> Option<Trial>;
    fn evaluate(&mut self, decision: usize, target: usize) -> f64;
}

// ---------------------------------------------------------------------------
// V4 Training trial
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Trial {
    pub class: usize,
    pub start_tick: usize,
    pub presentation_ticks: usize,
    pub read_delay: usize,
    /// V5 : classe cible (peut différer de class pour inversé/remap)
    pub target_class: usize,
}

// ---------------------------------------------------------------------------
// V5.orch : TrialContext — extracted from step_gpu / step_cpu
// ---------------------------------------------------------------------------

/// Context computed once per tick from the trial schedule.
struct TrialContext {
    /// Stimulus class to inject (-1 = none)
    stimulus_class: i32,
    /// Whether to reset activations at trial start
    reset_activations: bool,
    /// Whether to perform readout this tick
    need_readout: bool,
    /// Whether we are currently in a stimulus window
    in_stimulus: bool,
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
    buf_source_contribs: Vec<f32>,
    buf_influences: Vec<f32>,
    buf_fired_list: Vec<usize>,
    buf_activation_ticks: Vec<Option<usize>>,
    buf_node_delta_dopa: Vec<f32>,
    cached_dopa_center: Option<[f64; 3]>,
    // --- Compute backend ---
    pub backend: ComputeBackend,
    // --- Monitoring ---
    pub perf: PerfMonitor,
    // --- V5 additions ---
    /// V5 : mapping cible (classe_input → classe_output)
    pub target_mapping: Vec<usize>,
    /// V5 : snapshot conductances initiales (pour CT)
    pub initial_conductances: Option<Vec<f32>>,
    /// V5 : buffer pour la réverbération (activations tick-1)
    buf_prev_activations: Vec<f32>,
    /// V5 : tracker de sustain ratio
    pub sustain_tracker: SustainTracker,
    /// V5 : plasticité désactivée (baseline topology-only)
    pub plasticity_disabled: bool,
    /// V6 : tick de première activation par neurone (u32::MAX = jamais)
    buf_first_activation_tick: Vec<u32>,
}

pub struct SimulationResult {
    pub metrics: MetricsLog,
    pub domain: Domain,
}

impl Simulation {
    pub fn new(config: SimConfig) -> Self {
        init_thread_pool();

        // V5 : appliquer la calibration multi-échelle AVANT la construction
        let mut config = config;
        calibration::apply_calibration(&mut config);

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

        // Attempt GPU initialization if backend = "auto" or "gpu"
        let backend_pref = config.backend_pref();
        let backend = match backend_pref {
            "cpu" => {
                eprintln!("[Stretch] Compute backend: CPU (forced)");
                ComputeBackend::Cpu
            }
            _ => {
                // "auto" or "gpu"
                match GpuContext::try_new(&domain, &config, &zone_manager) {
                    Some(gpu) => {
                        eprintln!("[Stretch] Compute backend: GPU");
                        ComputeBackend::Gpu(gpu)
                    }
                    None => {
                        if backend_pref == "gpu" {
                            eprintln!("[Stretch] WARNING: GPU requested but unavailable, falling back to CPU");
                        } else {
                            eprintln!("[Stretch] Compute backend: CPU (no GPU adapter found)");
                        }
                        ComputeBackend::Cpu
                    }
                }
            }
        };

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
            buf_fired_list: Vec::with_capacity(n_nodes / 20), // ~5% active expected
            buf_activation_ticks: vec![None; n_nodes],
            buf_node_delta_dopa: Vec::new(),
            cached_dopa_center: None,
            backend,
            perf,
            target_mapping: vec![0, 1], // default: identity mapping
            initial_conductances: None,
            buf_prev_activations: vec![0.0; n_nodes],
            sustain_tracker: SustainTracker::new(),
            plasticity_disabled: false,
            buf_first_activation_tick: vec![u32::MAX; n_nodes],
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
                target_class: class, // V4 legacy: target = input class
            });
        }

        let n_trials = trials.len();
        self.schedule_trials(trials);

        // Upload I/O groups to GPU if available
        if let ComputeBackend::Gpu(ref gpu) = self.backend {
            let stim_groups: Vec<Vec<usize>> = self.input_encoder.as_ref()
                .map(|e| e.groups.clone()).unwrap_or_default();
            let read_groups: Vec<Vec<usize>> = self.output_reader.as_ref()
                .map(|r| r.groups.clone()).unwrap_or_default();
            gpu.upload_io_groups(&stim_groups, &read_groups);

            // KdTree no longer needed (spatial queries done above), drop CPU-only structures
            self.domain.compact_for_gpu();
        } else {
            // CPU mode: only drop kdtree (not needed after spatial setup)
            self.domain.drop_kdtree();
        }

        n_trials
    }

    /// V5 : Configuration complète avec tâches anti-biais, baselines et calibration.
    pub fn setup_v5_training(&mut self) -> usize {
        let config = self.config.clone();
        let extent = config.domain.domain_extent;
        let group_size = config.input.group_size;
        let task_mode = &config.v5_task.task_mode;
        let baseline_mode = &config.v5_task.baseline_mode;

        // V5 : placement I/O selon le mode de tâche
        let placement = task::place_io(
            &self.domain,
            task_mode,
            config.input.num_classes,
            group_size,
            extent,
        );

        if let Some(ref mut encoder) = self.input_encoder {
            encoder.groups = placement.input_groups.clone();
        }
        if let Some(ref mut reader) = self.output_reader {
            reader.groups = placement.output_groups.clone();
        }
        // V5 : appliquer inversion du mapping si demandé (indépendant de la géométrie)
        let effective_mapping: Vec<usize> = if config.v5_task.invert_mapping {
            placement.target_mapping.iter().copied().rev().collect()
        } else {
            placement.target_mapping.clone()
        };
        self.target_mapping = effective_mapping.clone();

        // V5 : appliquer le mode baseline
        task::apply_baseline_mode(&mut self.domain, baseline_mode, config.simulation.seed);
        if *baseline_mode == V5BaselineMode::TopologyOnly
            || *baseline_mode == V5BaselineMode::RandomBaseline
        {
            self.plasticity_disabled = true;
        }

        // Présentation : configurable ou fallback V4
        let presentation_ticks = if config.v5_task.presentation_ticks > 0 {
            config.v5_task.presentation_ticks
        } else {
            5
        };
        let read_delay = config.output.read_delay;
        let inter_trial_gap = 15;
        let warmup_ticks = 20;

        let mut trials = task::generate_trials(
            config.input.num_classes,
            &effective_mapping,
            config.simulation.total_ticks,
            presentation_ticks,
            read_delay,
            warmup_ticks,
            inter_trial_gap,
        );

        // V5 Remap : inverser le mapping à mi-parcours
        if *task_mode == V5TaskMode::Remap {
            let inverted_mapping: Vec<usize> = (0..config.input.num_classes).rev().collect();
            task::remap_trials(&mut trials, config.v5_task.remap_at_tick, &inverted_mapping);
        }

        let n_trials = trials.len();
        self.schedule_trials(trials);

        // V5 : sauvegarder les conductances initiales (pour CT)
        if config.v5_diagnostics.topological_coherence {
            let initial: Vec<f32> = self.domain.edges.iter().map(|e| e.conductance).collect();
            self.initial_conductances = Some(initial);
        }

        // Compact / GPU upload
        if let ComputeBackend::Gpu(ref gpu) = self.backend {
            let stim_groups: Vec<Vec<usize>> = self.input_encoder.as_ref()
                .map(|e| e.groups.clone()).unwrap_or_default();
            let read_groups: Vec<Vec<usize>> = self.output_reader.as_ref()
                .map(|r| r.groups.clone()).unwrap_or_default();
            gpu.upload_io_groups(&stim_groups, &read_groups);
            self.domain.compact_for_gpu();
        } else {
            self.domain.drop_kdtree();
        }

        // V5 : log le mode de tâche
        eprintln!("[V5] task_mode={:?}, baseline={:?}, mapping={:?}",
            task_mode, baseline_mode, &self.target_mapping);

        n_trials
    }

    pub fn total_ticks(&self) -> usize {
        self.config.simulation.total_ticks
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_evaluated == 0 { 0.0 } else { self.correct_count as f64 / self.total_evaluated as f64 }
    }

    /// Build a lightweight snapshot for visualization.
    /// For GPU backend, downloads node state from GPU.
    /// Call once per render frame, not every sim tick.
    pub fn build_viz_snapshot(&self, positions: &Arc<Vec<[f64; 3]>>, is_excitatory: &Arc<Vec<bool>>) -> VizSnapshot {
        // Get per-node data (GPU readback or CPU direct)
        let (activations, memory_traces, fatigues) = match &self.backend {
            ComputeBackend::Gpu(gpu) => {
                let gpu_nodes = gpu.download_nodes();
                let acts: Vec<f32> = gpu_nodes.iter().map(|n| n.activation).collect();
                let traces: Vec<f32> = gpu_nodes.iter().map(|n| n.memory_trace).collect();
                let fats: Vec<f32> = gpu_nodes.iter().map(|n| n.fatigue).collect();
                (acts, traces, fats)
            }
            ComputeBackend::Cpu => {
                let acts: Vec<f32> = self.domain.nodes.iter().map(|n| n.activation).collect();
                let traces: Vec<f32> = self.domain.nodes.iter().map(|n| n.memory_trace).collect();
                let fats: Vec<f32> = self.domain.nodes.iter().map(|n| n.fatigue).collect();
                (acts, traces, fats)
            }
        };

        // Active indices (for culling)
        let active_indices: Vec<usize> = activations.iter().enumerate()
            .filter(|(_, &a)| a > 0.01)
            .map(|(i, _)| i)
            .collect();

        // Sidebar metrics from last TickMetrics snapshot
        let last_tm = self.metrics.snapshots.last();
        let metrics = VizMetrics {
            active_count: last_tm.map_or(0, |m| m.active_nodes),
            total_energy: last_tm.map_or(0.0, |m| m.global_energy),
            max_activation: last_tm.map_or(0.0, |m| m.max_activation),
            mean_conductance: last_tm.map_or(0.0, |m| m.mean_conductance),
            max_conductance: last_tm.map_or(0.0, |m| m.max_conductance),
            mean_trace: last_tm.map_or(0.0, |m| m.mean_memory_trace),
            max_trace: last_tm.map_or(0.0, |m| m.max_memory_trace),
            mean_fatigue: last_tm.map_or(0.0, |m| m.mean_fatigue),
            dopamine_level: self.dopamine_system.level,
            current_trial: self.current_trial,
            total_trials: self.trials.len(),
            correct_count: self.correct_count,
            total_evaluated: self.total_evaluated,
            accuracy: self.accuracy() as f32,
            last_decision: self.last_decision,
            last_target: self.last_target,
            consolidated_edges: last_tm.map_or(0, |m| m.consolidated_edges),
            excitatory_energy: last_tm.map_or(0.0, |m| m.excitatory_energy),
            inhibitory_energy: last_tm.map_or(0.0, |m| m.inhibitory_energy),
            active_excitatory: last_tm.map_or(0, |m| m.active_excitatory),
            active_inhibitory: last_tm.map_or(0, |m| m.active_inhibitory),
            num_zones: self.zone_manager.num_zones(),
            zone_activity_mean: self.zone_manager.global_activity_mean(),
            mean_pid_error: self.zone_manager.mean_pid_error(),
            mean_pid_output: self.zone_manager.mean_pid_output(),
            pid_mode: self.config.zones.pid_mode.clone(),
            v6_sparsity_enabled: self.config.v6_sparsity.enabled,
            v6_dopa_mod_enabled: self.config.v6_dopa_modulation.enabled,
            v6_novelty_active: self.buf_first_activation_tick.iter().filter(|&&t| t < u32::MAX).count(),
        };

        // Energy sparkline history
        let n_points = self.metrics.snapshots.len().min(80);
        let start = self.metrics.snapshots.len() - n_points;
        let energy_history: Vec<f32> = self.metrics.snapshots[start..]
            .iter()
            .map(|s| s.global_energy)
            .collect();

        VizSnapshot {
            tick: self.tick,
            finished: self.finished,
            positions: Arc::clone(positions),
            activations,
            memory_traces,
            fatigues,
            is_excitatory: Arc::clone(is_excitatory),
            active_indices,
            metrics,
            energy_history,
        }
    }

    // -----------------------------------------------------------------------
    // V5.orch : shared trial context & readout processing
    // -----------------------------------------------------------------------

    /// Compute trial context for the current tick (shared GPU/CPU).
    fn compute_trial_context(&self, tick: usize) -> TrialContext {
        let mut ctx = TrialContext {
            stimulus_class: -1,
            reset_activations: false,
            need_readout: false,
            in_stimulus: false,
        };
        if self.current_trial < self.trials.len() {
            let trial = &self.trials[self.current_trial];
            if tick == trial.start_tick {
                ctx.reset_activations = true;
            }
            if tick >= trial.start_tick && tick < trial.start_tick + trial.presentation_ticks {
                ctx.stimulus_class = trial.class as i32;
                ctx.in_stimulus = true;
            }
            let read_tick = trial.start_tick + trial.presentation_ticks + trial.read_delay;
            if tick == read_tick {
                ctx.need_readout = true;
            }
        }
        ctx
    }

    /// Process readout scores: decision, reward, dopamine, spatial center (shared GPU/CPU).
    fn process_readout(&mut self, tick: usize, scores: &[f32]) {
        let config = &self.config;
        let decision = scores.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.last_decision = Some(decision);

        // Compute margin for V5.2 margin modulation
        let margin: f32 = if scores.len() >= 2 {
            let mut sorted = scores.to_vec();
            sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            sorted[0] - sorted[1]
        } else {
            0.0
        };

        if self.total_evaluated > 0 && self.total_evaluated % 10 == 0 {
            eprintln!("  [t={} trial={}] target={} dec={} scores={:?} margin={:.1}",
                tick, self.current_trial, self.last_target.unwrap_or(99), decision, scores, margin);
        }

        if let Some(target) = self.last_target {
            self.total_evaluated += 1;
            let correct = decision == target;
            if correct { self.correct_count += 1; }

            let reward_val = if correct {
                config.reward.reward_positive
            } else {
                config.reward.reward_negative
            };

            // V5.2: margin modulation — attenuate reward for trivial trials
            let r_eff = if config.reward.margin_modulation && margin > 0.0 {
                reward_val / (1.0 + config.reward.margin_beta * margin.abs() as f64)
            } else {
                reward_val
            };

            self.reward_system.set_reward(r_eff);

            // V5.2: RPE — compute δ = r_eff − baseline
            let delta = if config.reward.rpe_enabled {
                self.reward_system.compute_rpe(r_eff as f32, config.reward.rpe_alpha as f32)
            } else {
                r_eff as f32
            };

            // Dopamine update uses δ (RPE) instead of raw reward
            self.dopamine_system.update(delta as f64, &config.dopamine);

            // Spatial dopamine focus
            if config.dopamine.spatial_lambda > 0.0 {
                let focus_group_idx = if correct { target } else { decision };
                if let Some(ref reader_ref) = self.output_reader {
                    let focus_group = &reader_ref.groups[focus_group_idx];
                    let n = focus_group.len() as f64;
                    if n > 0.0 {
                        let (mut cx, mut cy, mut cz) = (0.0_f64, 0.0_f64, 0.0_f64);
                        for &idx in focus_group {
                            let p = self.domain.positions[idx];
                            cx += p[0]; cy += p[1]; cz += p[2];
                        }
                        self.reward_center = Some([cx / n, cy / n, cz / n]);
                    }
                }
            } else {
                self.reward_center = None;
            }
        }

        self.current_trial += 1;
    }

    /// V4 : Exécuter un seul tick (GPU-first or CPU fallback).
    pub fn step(&mut self) -> TickMetrics {
        let tick = self.tick;

        self.perf.begin_tick();

        match &self.backend {
            ComputeBackend::Gpu(_) => self.step_gpu(tick),
            ComputeBackend::Cpu => self.step_cpu(tick),
        }

        // === Métriques ===
        let snap_interval = self.config.simulation.snapshot_interval.max(1);
        if tick % snap_interval == 0 {
            if let ComputeBackend::Gpu(ref gpu) = self.backend {
                // GPU path: fast metrics reduction (64 bytes readback instead of full state)
                let gm = gpu.compute_gpu_metrics();
                self.metrics.record_from_gpu(
                    tick,
                    &gm,
                    &self.zone_manager,
                    self.reward_system.cumulative,
                    self.dopamine_system.level,
                    self.last_decision,
                    self.accuracy(),
                    self.reward_system.baseline,
                    self.reward_system.rpe_delta,
                );
            } else {
                self.metrics.record(
                    tick,
                    &self.domain,
                    &self.zone_manager,
                    self.reward_system.cumulative,
                    self.dopamine_system.level,
                    self.last_decision,
                    self.accuracy(),
                    self.reward_system.baseline,
                    self.reward_system.rpe_delta,
                );
            }
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
            rpe_baseline: 0.0,
            rpe_delta: 0.0,
        });

        self.perf.end_phase(Phase::Metrics);
        self.perf.end_tick(tick);

        self.tick += 1;
        if self.config.simulation.total_ticks > 0 && self.tick >= self.config.simulation.total_ticks {
            self.finished = true;
        }

        tick_metrics
    }

    /// GPU-first path: entire pipeline runs on GPU in a single submit.
    fn step_gpu(&mut self, tick: usize) {
        let config = &self.config;
        let ctx = self.compute_trial_context(tick);

        // Update last_target during stimulus window
        if ctx.in_stimulus {
            if let Some(trial) = self.trials.get(self.current_trial) {
                self.last_target = Some(trial.target_class);
            }
        }

        // V6: Reset first_activation_tick at trial start
        if ctx.reset_activations && config.v6_sparsity.enabled {
            crate::sparsity::reset_first_activation_tick(&mut self.buf_first_activation_tick);
        }

        // --- Upload spatial dopamine if reward center changed ---
        if let Some(center) = self.reward_center {
            if self.cached_dopa_center.as_ref() != Some(&center) {
                stdp::precompute_spatial_dopamine(
                    &self.domain.positions,
                    center,
                    1.0,
                    config.dopamine.spatial_lambda,
                    &mut self.buf_node_delta_dopa,
                );
                self.cached_dopa_center = Some(center);
                if let ComputeBackend::Gpu(ref gpu) = self.backend {
                    gpu.upload_node_delta_dopa(&self.buf_node_delta_dopa);
                }
            }
        }

        // --- Build GpuParams ---
        let dopa_phasic = self.dopamine_system.level - self.dopamine_system.tonic;
        let gpu_params = GpuParams {
            num_nodes: self.domain.num_nodes() as u32,
            num_edges: self.domain.num_edges() as u32,
            current_tick: tick as u32,
            propagation_gain: config.propagation.gain as f32,
            gain_inhibitory: config.propagation.gain_inhibitory as f32,
            activation_decay: config.dissipation.activation_decay as f32,
            activation_min: config.dissipation.activation_min as f32,
            fatigue_gain: config.dissipation.fatigue_gain as f32,
            fatigue_recovery: config.dissipation.fatigue_recovery as f32,
            inhibition_gain: config.dissipation.inhibition_gain as f32,
            inhibition_decay: config.dissipation.inhibition_decay as f32,
            trace_gain: config.dissipation.trace_gain as f32,
            trace_decay: config.dissipation.trace_decay as f32,
            decay_jitter: config.dissipation.decay_jitter as f32,
            a_plus: config.stdp.a_plus as f32,
            a_minus: config.stdp.a_minus as f32,
            tau_plus: config.stdp.tau_plus as f32,
            tau_minus: config.stdp.tau_minus as f32,
            elig_decay: config.eligibility.decay as f32,
            elig_max: config.eligibility.max as f32,
            plasticity_gain: config.dopamine.plasticity_gain as f32,
            global_delta_dopa: dopa_phasic,
            dopa_phasic: dopa_phasic,
            use_spatial: if self.reward_center.is_some() { 1 } else { 0 },
            spatial_lambda: config.dopamine.spatial_lambda as f32,
            cond_min: config.edge_defaults.conductance_min as f32,
            cond_max: config.edge_defaults.conductance_max as f32,
            homeostatic_rate: config.edge_defaults.decay as f32,
            baseline_cond: config.edge_defaults.conductance as f32,
            dopamine_level: self.dopamine_system.level,
            dopa_consol_threshold: config.dopamine.consolidation_threshold as f32,
            consol_conductance_threshold: config.consolidation.threshold as f32,
            consol_ticks_required: config.consolidation.ticks_required as u32,
            budget: config.synaptic_budget.budget as f32,
            stimulus_class: ctx.stimulus_class,
            stimulus_intensity: config.input.intensity as f32,
            num_zones: config.zones.num_zones as u32,
            zone_kp: config.zones.kp as f32,
            zone_ki: config.zones.ki as f32,
            zone_kd: config.zones.kd as f32,
            zone_pid_output_max: config.zones.pid_output_max as f32,
            zone_pid_integral_max: config.zones.pid_integral_max as f32,
            zone_k_theta: config.zones.k_theta as f32,
            zone_k_gain: config.zones.k_gain as f32,
            stimulus_group_size: config.input.group_size as u32,
            // V5.2 fields
            reset_policy: match config.v5_sustained.reset_policy.as_str() {
                "partial" => 1,
                "none" => 2,
                _ => 0, // "full"
            },
            adaptive_decay_enabled: if config.v5_sustained.adaptive_decay { 1 } else { 0 },
            k_local: config.v5_sustained.k_local as f32,
            reverberation_enabled: if config.v5_sustained.reverberation { 1 } else { 0 },
            reverb_gain: config.v5_sustained.reverb_gain as f32,
            rpe_delta: self.reward_system.rpe_delta,
            rho_boost: config.reward.rho_boost as f32,
            plasticity_disabled: if self.plasticity_disabled { 1 } else { 0 },
            num_classes: config.input.num_classes as u32,
            // V6 fields
            sparsity_enabled: if config.v6_sparsity.enabled { 1 } else { 0 },
            max_active_count: ((self.domain.num_nodes() as f64) * config.v6_sparsity.max_active_fraction) as u32,
            suppress_factor: config.v6_sparsity.suppress_factor as f32,
            novelty_gain: config.v6_sparsity.novelty_gain as f32,
            novelty_window: config.v6_sparsity.novelty_window,
            dopa_mod_enabled: if config.v6_dopa_modulation.enabled { 1 } else { 0 },
            reverb_min: config.v6_dopa_modulation.reverb_min as f32,
            reverb_max: config.v6_dopa_modulation.reverb_max as f32,
            decay_mod_strength: config.v6_dopa_modulation.decay_mod_strength as f32,
            dopa_threshold: config.v6_dopa_modulation.dopa_threshold as f32,
            dopa_kappa: config.v6_dopa_modulation.dopa_kappa as f32,
            _pad_v6_0: 0,
            _pad_v6_1: 0,
            _pad_v6_2: 0,
        };

        // V6: Compute sparsity threshold on CPU (from previous tick's activations)
        let sparsity_threshold = if config.v6_sparsity.enabled {
            if let ComputeBackend::Gpu(ref gpu) = self.backend {
                crate::sparsity::compute_sparsity_threshold(
                    &gpu.read_activations_fast(self.domain.num_nodes()),
                    &self.buf_first_activation_tick,
                    tick as u32,
                    &config.v6_sparsity,
                )
            } else {
                0.0
            }
        } else {
            0.0
        };

        // --- Single GPU submit ---
        if let ComputeBackend::Gpu(ref gpu) = self.backend {
            gpu.run_full_tick(&gpu_params, ctx.need_readout, ctx.reset_activations, sparsity_threshold);
        }

        self.perf.end_phase(Phase::GpuTick);

        // --- Dopamine decay ---
        self.dopamine_system.update(0.0, &config.dopamine);

        // --- Readout + reward (shared) ---
        if ctx.need_readout {
            if let ComputeBackend::Gpu(ref gpu) = self.backend {
                let scores = gpu.read_readout_scores(config.input.num_classes);
                self.process_readout(tick, &scores);
            }
        }

        self.reward_system.clear();
        self.perf.end_phase(Phase::ReadoutReward);
    }

    /// CPU fallback path: original hybrid step logic (no GPU).
    fn step_cpu(&mut self, tick: usize) {
        let config = &self.config;
        let ctx = self.compute_trial_context(tick);

        // === Phase 0+1 : Zones + PID ===
        self.zone_manager.measure(&self.domain);
        self.zone_manager.regulate(&mut self.domain, &config.zones);

        self.perf.end_phase(Phase::Zones);

        // === Phase 2 : Stimuli + pacemakers ===
        if self.input_encoder.as_ref().map_or(true, |e| e.groups.is_empty()) {
            stimulus::inject_stimuli(&mut self.domain, &config.stimuli, tick);
        }
        if !config.pacemakers.is_empty() {
            pacemaker::apply_pacemakers(&mut self.domain, &config.pacemakers, tick);
        }

        // === Phase 3 : V4/V5 — injection d'entrée ===
        if let Some(ref encoder) = self.input_encoder {
            if ctx.reset_activations {
                // V5 : politique de reset configurable
                sustained::apply_reset_policy(&mut self.domain, &config.v5_sustained.reset_policy);
                // V6 : reset first_activation_tick
                if config.v6_sparsity.enabled {
                    crate::sparsity::reset_first_activation_tick(&mut self.buf_first_activation_tick);
                }
            }
            if ctx.in_stimulus {
                if let Some(trial) = self.trials.get(self.current_trial) {
                    encoder.inject(&mut self.domain, trial.class, config.input.intensity);
                    self.last_target = Some(trial.target_class);
                }
            }
        }

        // V5 : sustain ratio tracking
        self.sustain_tracker.set_in_stimulus(ctx.in_stimulus);

        self.perf.end_phase(Phase::StimInput);

        // === Phase 4 : Propagation (CPU only) ===
        propagation::compute_source_contribs(
            &self.domain,
            &config.propagation,
            &self.zone_manager.gain_mods,
            &mut self.buf_source_contribs,
            &mut self.buf_fired_list,
        );

        propagation::compute_influences_fired_only(
            &self.domain,
            &self.buf_source_contribs,
            &self.buf_fired_list,
            &mut self.buf_influences,
        );
        propagation::apply_influences(&mut self.domain, &self.buf_influences);

        self.perf.end_phase(Phase::Propagation);

        // === Phase 4a : V6 — Sparsité à front d'onde ===
        if config.v6_sparsity.enabled {
            let activations: Vec<f32> = self.domain.nodes.iter().map(|n| n.activation).collect();
            let thresholds: Vec<f32> = self.domain.nodes.iter().map(|n| n.threshold).collect();
            let fatigues: Vec<f32> = self.domain.nodes.iter().map(|n| n.fatigue).collect();
            let inhibitions: Vec<f32> = self.domain.nodes.iter().map(|n| n.inhibition).collect();
            let excitabilities: Vec<f32> = self.domain.nodes.iter().map(|n| n.excitability).collect();
            let threshold_mods: Vec<f32> = self.domain.nodes.iter().map(|n| n.threshold_mod).collect();

            let threshold = crate::sparsity::compute_sparsity_threshold(
                &activations,
                &self.buf_first_activation_tick,
                tick as u32,
                &config.v6_sparsity,
            );

            let mut cpu_activations: Vec<f32> = activations;
            crate::sparsity::apply_sparsity_cpu(
                &mut cpu_activations,
                &mut self.buf_first_activation_tick,
                &thresholds,
                &fatigues,
                &inhibitions,
                &excitabilities,
                &threshold_mods,
                tick as u32,
                &config.v6_sparsity,
                threshold,
            );

            // Write back activations
            for (node, &act) in self.domain.nodes.iter_mut().zip(cpu_activations.iter()) {
                node.activation = act;
            }
        }

        // === Phase 4b : V5 — Réverbération locale (avant dissipation) ===
        if config.v5_sustained.reverberation && config.v5_sustained.reverb_gain > 0.0 {
            sustained::apply_reverberation(
                &mut self.domain,
                &self.buf_prev_activations,
                config.v5_sustained.reverb_gain as f32,
            );
        }

        // V5 : snapshot activations pour la réverbération du tick suivant
        if config.v5_sustained.reverberation {
            sustained::snapshot_activations(&self.domain, &mut self.buf_prev_activations);
        }

        // V5 : enregistrer l'énergie pour le sustain ratio
        if config.v5_diagnostics.sustain_ratio {
            let energy: f64 = self.domain.nodes.iter().map(|n| n.activation as f64).sum();
            self.sustain_tracker.record_energy(energy);
        }

        // === Phase 5 : Dissipation parallèle ===
        if config.v5_sustained.adaptive_decay {
            // V5 : decay adaptatif
            sustained::apply_adaptive_decay(
                &mut self.domain,
                config.dissipation.activation_decay as f32,
                config.v5_sustained.k_local as f32,
                config.dissipation.activation_min as f32,
            );
            // Still need fatigue/inhibition/trace updates
            let fatigue_gain = config.dissipation.fatigue_gain as f32;
            let fatigue_recovery = config.dissipation.fatigue_recovery as f32;
            let inhibition_gain = config.dissipation.inhibition_gain as f32;
            let inhibition_decay_rate = config.dissipation.inhibition_decay as f32;
            let trace_gain = config.dissipation.trace_gain as f32;
            let trace_decay = config.dissipation.trace_decay as f32;
            self.domain.nodes.par_iter_mut()
                .zip(self.domain.node_needs_update.par_iter_mut())
                .for_each(|(node, needs_update)| {
                    if !*needs_update && !node.is_active()
                        && node.fatigue < 0.01 && node.inhibition < 0.01
                    { return; }
                    node.update_fatigue(fatigue_gain, fatigue_recovery);
                    node.update_inhibition(inhibition_gain, inhibition_decay_rate);
                    node.update_trace(trace_gain, trace_decay);
                    node.update_excitability_from_trace();
                    *needs_update = node.is_active() || node.fatigue > 0.01 || node.inhibition > 0.01;
                });
        } else {
            // V4 classique : decay fixe avec jitter
            let base_decay = config.dissipation.activation_decay as f32;
            let jitter = config.dissipation.decay_jitter as f32;
            let activation_min = config.dissipation.activation_min as f32;
            let fatigue_gain = config.dissipation.fatigue_gain as f32;
            let fatigue_recovery = config.dissipation.fatigue_recovery as f32;
            let inhibition_gain = config.dissipation.inhibition_gain as f32;
            let inhibition_decay_rate = config.dissipation.inhibition_decay as f32;
            let trace_gain = config.dissipation.trace_gain as f32;
            let trace_decay = config.dissipation.trace_decay as f32;
            let jitter_seed = tick as u64;

            self.domain
                .nodes
                .par_iter_mut()
                .zip(self.domain.node_needs_update.par_iter_mut())
                .enumerate()
                .for_each(|(i, (node, needs_update))| {
                    if !*needs_update && !node.is_active()
                        && node.fatigue < 0.01 && node.inhibition < 0.01
                    {
                        return;
                    }

                    let effective_decay = if jitter > 0.0 {
                        let h = (i as u64)
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(jitter_seed)
                            .wrapping_mul(1442695040888963407);
                        let j = ((h >> 33) as f32 / (1u64 << 31) as f32 - 1.0) * jitter;
                        (base_decay * (1.0 + j)).clamp(0.0, 1.0)
                    } else {
                        base_decay
                    };
                    node.update_fatigue(fatigue_gain, fatigue_recovery);
                    node.update_inhibition(inhibition_gain, inhibition_decay_rate);
                    node.update_trace(trace_gain, trace_decay);
                    node.update_excitability_from_trace();
                    node.decay_activation(effective_decay, activation_min);

                    *needs_update = node.is_active()
                        || node.fatigue > 0.01
                        || node.inhibition > 0.01;
                });
        } // end else (V4 dissipation)

        self.perf.end_phase(Phase::Dissipation);

        // === Phase 5b : Décroissance naturelle dopamine ===
        self.dopamine_system.update(0.0, &config.dopamine);

        // === Phase 6 : Plasticité (CPU) — V5 : désactivable pour baselines ===
        if !self.plasticity_disabled {
            let dopamine_level = self.dopamine_system.level as f64;
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
                self.reward_system.rpe_delta,
                config.reward.rho_boost as f32,
                &mut self.buf_activation_ticks,
                &mut self.buf_node_delta_dopa,
                &mut self.cached_dopa_center,
            );
        }

        self.domain.sync_conductances();

        self.perf.end_phase(Phase::Plasticity);

        // === Phase 7 : V4 — readout de sortie (shared) ===
        if ctx.need_readout {
            if let Some(ref reader) = self.output_reader {
                let result = reader.readout(&self.domain);
                self.process_readout(tick, &result.scores);
            }
        }

        self.reward_system.clear();

        self.perf.end_phase(Phase::ReadoutReward);
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

    // Final GPU → CPU sync for accurate end-of-simulation stats
    if let ComputeBackend::Gpu(ref gpu) = sim.backend {
        gpu.sync_state_to_domain(&mut sim.domain);
    }

    let active_final = sim.domain.nodes.iter().filter(|n| n.is_active()).count();
    let energy_final: f32 = sim.domain.nodes.iter().map(|n| n.activation).sum();
    let max_trace = sim.domain.nodes.iter().map(|n| n.memory_trace).fold(0.0_f32, f32::max);
    let max_cond = sim.domain.edges.iter().map(|e| e.conductance).fold(0.0_f32, f32::max);

    println!("\n=== Fin simulation ===");
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

/// Exécution d'une simulation déjà configurée (avec trials pré-programmés).
/// Contrairement à run_with_observer, ne crée pas de Simulation — utilise celle fournie.
pub fn run_simulation_loop(mut sim: Simulation, observer: &mut dyn SimulationObserver) -> SimulationResult {
    let t_start = std::time::Instant::now();
    let config = sim.config.clone();

    observer.on_init(&sim.domain, &config);

    let ticks_label = if sim.total_ticks() == 0 { "∞".to_string() } else { sim.total_ticks().to_string() };
    let is_v5 = config.v5_task.task_mode != crate::config::V5TaskMode::Legacy;
    let version_label = if is_v5 {
        format!("V5 [{:?}/{:?}]", config.v5_task.task_mode, config.v5_task.baseline_mode)
    } else {
        "V4 [dopamine+reward]".to_string()
    };
    println!(
        "=== Simulation {} ===\nTopologie: {} | Nœuds: {}, Liaisons: {}, Ticks: {} | Zones: {}",
        version_label,
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
        if !keep_going { break; }

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
                t_start.elapsed().as_secs_f64() * 1000.0 / (tick + 1) as f64
            );
        }
    }

    // Final GPU → CPU sync for accurate end-of-simulation stats
    if let ComputeBackend::Gpu(ref gpu) = sim.backend {
        gpu.sync_state_to_domain(&mut sim.domain);
    }

    let active_final = sim.domain.nodes.iter().filter(|n| n.is_active()).count();
    let energy_final: f32 = sim.domain.nodes.iter().map(|n| n.activation).sum();
    println!("\n=== Fin simulation {} ===", version_label);
    println!("  Nœuds actifs finaux : {}", active_final);
    println!("  Énergie finale      : {:.4}", energy_final);
    if sim.total_evaluated > 0 {
        println!("  Accuracy finale     : {:.1}% ({}/{})", sim.accuracy() * 100.0, sim.correct_count, sim.total_evaluated);
    }
    println!("  Dopamine finale     : {:.4}", sim.dopamine_system.level);
    println!("  Récompense cumulée  : {:.4}", sim.reward_system.cumulative);

    observer.on_finish(&sim.domain, &sim.metrics);
    sim.into_result()
}

pub fn run(config: &SimConfig) -> SimulationResult {
    run_with_observer(config, &mut NullObserver)
}
