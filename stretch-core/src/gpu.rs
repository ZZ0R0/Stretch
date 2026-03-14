use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::config::SimConfig;
use crate::domain::Domain;
use crate::zone::ZoneManager;

// ============================================================================
// GPU Data Structures — must match WGSL struct layouts exactly
// ============================================================================

/// GPU node state (48 bytes, matches GpuNode in all shaders).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuNode {
    pub activation: f32,
    pub threshold: f32,
    pub fatigue: f32,
    pub memory_trace: f32,
    pub excitability: f32,
    pub inhibition: f32,
    pub threshold_mod: f32,
    pub last_activation_tick: i32,
    pub activation_count: u32,
    pub is_excitatory: u32,
    pub gain_mod: f32,
    pub _pad: f32,
}

/// GPU zone state (48 bytes, matches GpuZone in zones.wgsl).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuZone {
    pub target_activity: f32,
    pub activity_sum: f32,
    pub member_count: u32,
    pub error: f32,
    pub integral: f32,
    pub error_prev: f32,
    pub output: f32,
    pub theta_mod: f32,
    pub gain_mod: f32,
    pub stable_ticks: u32,
    pub is_stable: u32,
    pub _pad: f32,
}

/// GPU uniform parameters (192 bytes, matches GpuParams in all shaders).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuParams {
    // --- Sizes ---
    pub num_nodes: u32,
    pub num_edges: u32,
    pub current_tick: u32,
    // --- Propagation ---
    pub propagation_gain: f32,
    pub gain_inhibitory: f32,
    // --- Dissipation ---
    pub activation_decay: f32,
    pub activation_min: f32,
    pub fatigue_gain: f32,
    pub fatigue_recovery: f32,
    pub inhibition_gain: f32,
    pub inhibition_decay: f32,
    pub trace_gain: f32,
    pub trace_decay: f32,
    pub decay_jitter: f32,
    // --- STDP / Plasticity ---
    pub a_plus: f32,
    pub a_minus: f32,
    pub tau_plus: f32,
    pub tau_minus: f32,
    pub elig_decay: f32,
    pub elig_max: f32,
    pub plasticity_gain: f32,
    pub global_delta_dopa: f32,
    pub dopa_phasic: f32,
    pub use_spatial: u32,
    pub spatial_lambda: f32,
    pub cond_min: f32,
    pub cond_max: f32,
    pub homeostatic_rate: f32,
    pub baseline_cond: f32,
    pub dopamine_level: f32,
    pub dopa_consol_threshold: f32,
    pub consol_conductance_threshold: f32,
    pub consol_ticks_required: u32,
    pub budget: f32,
    // --- Stimulus ---
    pub stimulus_class: i32,
    pub stimulus_intensity: f32,
    // --- Zone PID ---
    pub num_zones: u32,
    pub zone_kp: f32,
    pub zone_ki: f32,
    pub zone_kd: f32,
    pub zone_pid_output_max: f32,
    pub zone_pid_integral_max: f32,
    pub zone_k_theta: f32,
    pub zone_k_gain: f32,
    // --- Groups ---
    pub stimulus_group_size: u32,
    // --- Padding to 192 bytes (16-byte aligned) ---
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

/// Edge data layout for GPU buffers (32 bytes).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuEdge {
    pub from: u32,
    pub to: u32,
    pub conductance: f32,
    pub eligibility: f32,
    pub consolidated: u32,
    pub consolidation_counter: u32,
    pub distance: f32,
    pub _pad: f32,
}

/// GPU metrics output — 16 × u32 = 64 bytes.
/// Decoded on CPU after readback.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuMetricsRaw {
    pub active_count: u32,
    pub sum_energy_fixed: u32,
    pub max_activation_bits: u32,
    pub sum_trace_fixed: u32,
    pub max_trace_bits: u32,
    pub sum_fatigue_fixed: u32,
    pub active_excitatory: u32,
    pub active_inhibitory: u32,
    pub sum_exc_energy_fixed: u32,
    pub sum_inh_energy_fixed: u32,
    pub sum_conductance_fixed: u32,
    pub max_conductance_bits: u32,
    pub consolidated_count: u32,
    pub sum_eligibility_fixed: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

/// Decoded GPU metrics.
pub struct GpuMetrics {
    pub active_count: u32,
    pub global_energy: f32,
    pub max_activation: f32,
    pub mean_memory_trace: f32,
    pub max_memory_trace: f32,
    pub mean_fatigue: f32,
    pub active_excitatory: u32,
    pub active_inhibitory: u32,
    pub excitatory_energy: f32,
    pub inhibitory_energy: f32,
    pub mean_conductance: f32,
    pub max_conductance: f32,
    pub consolidated_count: u32,
    pub mean_eligibility: f32,
}

// ============================================================================
// GPU-First Compute Context
// ============================================================================

#[allow(dead_code)]
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    // --- Pipelines ---
    reset_pipeline: wgpu::ComputePipeline,
    injection_pipeline: wgpu::ComputePipeline,
    zone_measure_pipeline: wgpu::ComputePipeline,
    zone_pid_pipeline: wgpu::ComputePipeline,
    zone_apply_pipeline: wgpu::ComputePipeline,
    source_contribs_pipeline: wgpu::ComputePipeline,
    propagation_pipeline: wgpu::ComputePipeline,
    apply_dissipate_pipeline: wgpu::ComputePipeline,
    plasticity_pipeline: wgpu::ComputePipeline,
    budget_sum_pipeline: wgpu::ComputePipeline,
    budget_scale_pipeline: wgpu::ComputePipeline,
    sync_conductances_pipeline: wgpu::ComputePipeline,
    readout_pipeline: wgpu::ComputePipeline,
    metrics_nodes_pipeline: wgpu::ComputePipeline,
    metrics_edges_pipeline: wgpu::ComputePipeline,
    // --- Persistent GPU buffers ---
    buf_nodes: wgpu::Buffer,
    buf_edges: wgpu::Buffer,
    buf_conductances: wgpu::Buffer,
    buf_source_contribs: wgpu::Buffer,
    buf_influences: wgpu::Buffer,
    buf_csr_offsets: wgpu::Buffer,
    buf_csr_sources: wgpu::Buffer,
    buf_csr_kernels: wgpu::Buffer,
    buf_csr_edge_indices: wgpu::Buffer,
    buf_node_delta_dopa: wgpu::Buffer,
    buf_params: wgpu::Buffer,
    buf_budget_totals: wgpu::Buffer,
    buf_stimulus_groups: wgpu::Buffer,
    buf_zone_assignments: wgpu::Buffer,
    buf_zone_state: wgpu::Buffer,
    buf_zone_accum: wgpu::Buffer,
    buf_readout_groups: wgpu::Buffer,
    buf_readout_scores: wgpu::Buffer,
    buf_metrics_output: wgpu::Buffer,
    // --- Staging buffers ---
    staging_readout: wgpu::Buffer,
    staging_edges: wgpu::Buffer,
    staging_nodes: wgpu::Buffer,
    staging_metrics: wgpu::Buffer,
    // --- Bind groups ---
    injection_bg: wgpu::BindGroup,
    zone_bg: wgpu::BindGroup,
    source_bg: wgpu::BindGroup,
    propagation_bg: wgpu::BindGroup,
    dissipate_bg: wgpu::BindGroup,
    plasticity_bg: wgpu::BindGroup,
    budget_bg: wgpu::BindGroup,
    sync_cond_bg: wgpu::BindGroup,
    readout_bg: wgpu::BindGroup,
    metrics_bg: wgpu::BindGroup,
    // --- Timestamp profiling ---
    timestamp_query_set: Option<wgpu::QuerySet>,
    timestamp_resolve_buf: Option<wgpu::Buffer>,
    timestamp_staging_buf: Option<wgpu::Buffer>,
    timestamp_period: f32,
    // --- Sizes ---
    pub num_nodes: u32,
    pub num_edges: u32,
    pub num_zones: u32,
    pub num_classes: u32,
    pub group_size: u32,
    workgroup_size: u32,
}

impl GpuContext {
    /// Split a 1D dispatch count into (x, y) where both ≤ 65535.
    /// GPU max workgroups per dimension is 65535.
    fn dispatch_2d(total_groups: u32) -> (u32, u32) {
        if total_groups <= 65535 {
            (total_groups, 1)
        } else {
            let x = 65535u32;
            let y = (total_groups + x - 1) / x;
            (x, y)
        }
    }

    /// Initialize GPU-first context. Returns None if no GPU adapter available.
    pub fn try_new(domain: &Domain, config: &SimConfig, zone_manager: &ZoneManager) -> Option<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))?;

        eprintln!("[GPU] Adapter: {:?}", adapter.get_info().name);

        // Request timestamp query feature if available
        let adapter_features = adapter.features();
        let mut features = wgpu::Features::empty();
        let has_timestamps = adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY);
        if has_timestamps {
            features |= wgpu::Features::TIMESTAMP_QUERY;
        }

        // Request adapter's actual limits (not conservative defaults)
        // This unlocks larger buffer sizes on capable GPUs.
        let adapter_limits = adapter.limits();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("stretch-gpu"),
                required_features: features,
                required_limits: adapter_limits.clone(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )).ok()?;

        let num_nodes = domain.num_nodes() as u32;
        let num_edges = domain.num_edges() as u32;

        // Pre-flight: check that the largest buffers fit within device limits
        let node_buf_size = (num_nodes as u64) * (std::mem::size_of::<GpuNode>() as u64);
        let edge_buf_size = (num_edges as u64) * (std::mem::size_of::<GpuEdge>() as u64);
        let max_buf = adapter_limits.max_buffer_size as u64;
        let max_binding = adapter_limits.max_storage_buffer_binding_size as u64;
        let largest = node_buf_size.max(edge_buf_size);
        if largest > max_buf || largest > max_binding {
            eprintln!(
                "[GPU] Network too large for GPU: need {} MB, device max buffer {} MB / max binding {} MB",
                largest / (1024 * 1024),
                max_buf / (1024 * 1024),
                max_binding / (1024 * 1024),
            );
            eprintln!("[GPU] Falling back to CPU.");
            return None;
        }
        let num_zones = config.zones.num_zones.max(1) as u32;
        let num_classes = config.input.num_classes as u32;
        let group_size = config.input.group_size as u32;
        let workgroup_size = 256u32;

        // ====================================================================
        // Create GPU buffers (each temp Vec dropped immediately after upload)
        // ====================================================================

        // Node state → GpuNode → buffer
        let buf_nodes = {
            let gpu_nodes: Vec<GpuNode> = domain.nodes.iter().enumerate().map(|(i, n)| GpuNode {
                activation: n.activation,
                threshold: n.threshold,
                fatigue: n.fatigue,
                memory_trace: n.memory_trace,
                excitability: n.excitability,
                inhibition: n.inhibition,
                threshold_mod: n.threshold_mod,
                last_activation_tick: n.last_activation_tick.map_or(-1, |t| t as i32),
                activation_count: n.activation_count as u32,
                is_excitatory: if domain.node_is_excitatory[i] { 1 } else { 0 },
                gain_mod: 0.0,
                _pad: 0.0,
            }).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("nodes"),
                contents: bytemuck::cast_slice(&gpu_nodes),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            })
        };

        // Edge state → GpuEdge → buffer
        let buf_edges = {
            let gpu_edges: Vec<GpuEdge> = domain.edges.iter().map(|e| GpuEdge {
                from: e.from as u32,
                to: e.to as u32,
                conductance: e.conductance,
                eligibility: e.eligibility,
                consolidated: if e.consolidated { 1 } else { 0 },
                consolidation_counter: e.consolidation_counter as u32,
                distance: e.distance,
                _pad: 0.0,
            }).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("edges"),
                contents: bytemuck::cast_slice(&gpu_edges),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            })
        };

        // CSR-ordered conductances → buffer
        let buf_conductances = {
            let csr_conductances: Vec<f32> = domain.incoming.edge_indices.iter()
                .map(|&i| domain.conductances[i])
                .collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("conductances"),
                contents: bytemuck::cast_slice(&csr_conductances),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        // CSR offsets → buffer
        let buf_csr_offsets = {
            let csr_offsets: Vec<u32> = domain.incoming.offsets.iter().map(|&x| x as u32).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("csr_offsets"),
                contents: bytemuck::cast_slice(&csr_offsets),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        // CSR sources → buffer
        let buf_csr_sources = {
            let csr_sources: Vec<u32> = domain.incoming.source_nodes.iter().map(|&x| x as u32).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("csr_sources"),
                contents: bytemuck::cast_slice(&csr_sources),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        // CSR kernels → buffer
        let buf_csr_kernels = {
            let csr_kernels: Vec<f32> = domain.incoming.kernel_weights.clone();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("csr_kernels"),
                contents: bytemuck::cast_slice(&csr_kernels),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        // CSR edge indices → buffer
        let buf_csr_edge_indices = {
            let csr_edge_indices: Vec<u32> = domain.incoming.edge_indices.iter().map(|&x| x as u32).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("csr_edge_indices"),
                contents: bytemuck::cast_slice(&csr_edge_indices),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let buf_source_contribs = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("source_contribs"),
            size: (num_nodes as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let buf_influences = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("influences"),
            size: (num_nodes as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let buf_node_delta_dopa = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("node_delta_dopa"),
            size: (num_nodes as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buf_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("params"),
            size: std::mem::size_of::<GpuParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buf_budget_totals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("budget_totals"),
            size: (num_nodes as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Stimulus/readout group buffers (filled later via upload_io_groups)
        let max_group_entries = (num_classes * group_size).max(1) as u64;
        let buf_stimulus_groups = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stimulus_groups"),
            size: max_group_entries * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buf_readout_groups = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readout_groups"),
            size: max_group_entries * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Zone assignments → buffer
        let buf_zone_assignments = {
            let zone_assignments: Vec<u32> = zone_manager.assignments.iter().map(|&x| x as u32).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zone_assignments"),
                contents: bytemuck::cast_slice(&zone_assignments),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        // Zone state → buffer
        let buf_zone_state = {
            let gpu_zones: Vec<GpuZone> = zone_manager.zones.iter().map(|z| GpuZone {
                target_activity: z.target_activity,
                activity_sum: z.activity_mean,
                member_count: z.members.len() as u32,
                error: z.error,
                integral: z.integral,
                error_prev: z.error_prev,
                output: z.output,
                theta_mod: z.theta_mod,
                gain_mod: z.gain_mod,
                stable_ticks: z.stable_ticks as u32,
                is_stable: if z.is_stable { 1 } else { 0 },
                _pad: 0.0,
            }).collect();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zone_state"),
                contents: bytemuck::cast_slice(&gpu_zones),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let buf_zone_accum = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zone_accum"),
            size: (num_zones as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Readout score buffer (atomic accumulators)
        let buf_readout_scores = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readout_scores"),
            size: (num_classes.max(1) as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Staging buffers for readback
        let staging_readout = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_readout"),
            size: (num_classes.max(1) as u64) * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Metrics output buffer (16 × u32 = 64 bytes)
        let metrics_size = std::mem::size_of::<GpuMetricsRaw>() as u64;
        let buf_metrics_output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metrics_output"),
            size: metrics_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let staging_metrics = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_metrics"),
            size: metrics_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_edges = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_edges"),
            size: (num_edges as u64) * std::mem::size_of::<GpuEdge>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_nodes = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_nodes"),
            size: (num_nodes as u64) * std::mem::size_of::<GpuNode>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Timestamp query resources (24 timestamps: begin+end for 12 phases)
        const TS_COUNT: u32 = 24;
        let (timestamp_query_set, timestamp_resolve_buf, timestamp_staging_buf) = if has_timestamps {
            let qs = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("perf_timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: TS_COUNT,
            });
            let resolve = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ts_resolve"),
                size: (TS_COUNT as u64) * 8, // u64 per timestamp
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let staging = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ts_staging"),
                size: (TS_COUNT as u64) * 8,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            (Some(qs), Some(resolve), Some(staging))
        } else {
            (None, None, None)
        };
        let timestamp_period = queue.get_timestamp_period();

        // ====================================================================
        // Load shaders
        // ====================================================================

        let injection_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("injection"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/injection.wgsl").into()),
        });
        let zones_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zones"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/zones.wgsl").into()),
        });
        let source_contribs_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("source_contribs"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/source_contribs.wgsl").into()),
        });
        let propagation_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("propagation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/propagation.wgsl").into()),
        });
        let apply_dissipate_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("apply_and_dissipate"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/apply_and_dissipate.wgsl").into()),
        });
        let plasticity_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("plasticity"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/plasticity.wgsl").into()),
        });
        let budget_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("budget"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/budget.wgsl").into()),
        });
        let sync_cond_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sync_conductances"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sync_conductances.wgsl").into()),
        });
        let readout_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("readout"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/readout.wgsl").into()),
        });
        let metrics_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("metrics_reduce"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/metrics_reduce.wgsl").into()),
        });

        // ====================================================================
        // Bind group layouts
        // ====================================================================

        // injection_bgl: nodes(rw), stimulus_groups(r), params(uniform)
        let injection_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("injection_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Uniform),
            ],
        });

        // zone_bgl: nodes(rw), zones(rw), zone_assignments(r), params(uniform), zone_accum(rw)
        let zone_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zone_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(3, wgpu::BufferBindingType::Uniform),
                bgl_entry(4, wgpu::BufferBindingType::Storage { read_only: false }),
            ],
        });

        // source_bgl: nodes(r), source_contribs(rw), params(uniform)
        let source_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("source_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(2, wgpu::BufferBindingType::Uniform),
            ],
        });

        // propagation_bgl: source_contribs(r), conductances(r), csr_offsets(r),
        //                   csr_sources(r), csr_kernels(r), influences(rw), params(uniform)
        let propagation_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("propagation_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(3, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(4, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(5, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(6, wgpu::BufferBindingType::Uniform),
            ],
        });

        // dissipate_bgl: nodes(rw), influences(r), params(uniform)
        let dissipate_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("dissipate_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Uniform),
            ],
        });

        // plasticity_bgl: edges(rw), nodes(r), node_delta_dopa(r), params(uniform)
        let plasticity_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("plasticity_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(3, wgpu::BufferBindingType::Uniform),
            ],
        });

        // budget_bgl: edges(rw), budget_totals(rw), params(uniform)
        let budget_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("budget_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(2, wgpu::BufferBindingType::Uniform),
            ],
        });

        // sync_cond_bgl: edges(r), csr_edge_indices(r), conductances(rw), params(uniform)
        let sync_cond_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sync_cond_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(3, wgpu::BufferBindingType::Uniform),
            ],
        });

        // readout_bgl: nodes(r), readout_groups(r), readout_scores(rw), params(uniform)
        let readout_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("readout_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(3, wgpu::BufferBindingType::Uniform),
            ],
        });

        // metrics_bgl: nodes(r), edges(r), metrics_output(rw), params(uniform)
        let metrics_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("metrics_bgl"),
            entries: &[
                bgl_entry(0, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                bgl_entry(2, wgpu::BufferBindingType::Storage { read_only: false }),
                bgl_entry(3, wgpu::BufferBindingType::Uniform),
            ],
        });

        // ====================================================================
        // Pipelines
        // ====================================================================

        let make_pipeline = |label: &str, layout: &wgpu::BindGroupLayout, module: &wgpu::ShaderModule, entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[layout],
                    push_constant_ranges: &[],
                })),
                module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };

        let reset_pipeline = make_pipeline("reset", &injection_bgl, &injection_shader, "reset_activations");
        let injection_pipeline = make_pipeline("injection", &injection_bgl, &injection_shader, "inject");
        let zone_measure_pipeline = make_pipeline("zone_measure", &zone_bgl, &zones_shader, "zone_measure");
        let zone_pid_pipeline = make_pipeline("zone_pid", &zone_bgl, &zones_shader, "zone_pid");
        let zone_apply_pipeline = make_pipeline("zone_apply", &zone_bgl, &zones_shader, "zone_apply");
        let source_contribs_pipeline = make_pipeline("source_contribs", &source_bgl, &source_contribs_shader, "main");
        let propagation_pipeline = make_pipeline("propagation", &propagation_bgl, &propagation_shader, "main");
        let apply_dissipate_pipeline = make_pipeline("apply_dissipate", &dissipate_bgl, &apply_dissipate_shader, "main");
        let plasticity_pipeline = make_pipeline("plasticity", &plasticity_bgl, &plasticity_shader, "main");
        let budget_sum_pipeline = make_pipeline("budget_sum", &budget_bgl, &budget_shader, "budget_sum");
        let budget_scale_pipeline = make_pipeline("budget_scale", &budget_bgl, &budget_shader, "budget_scale");
        let sync_conductances_pipeline = make_pipeline("sync_cond", &sync_cond_bgl, &sync_cond_shader, "main");
        let readout_pipeline = make_pipeline("readout", &readout_bgl, &readout_shader, "main");
        let metrics_nodes_pipeline = make_pipeline("metrics_nodes", &metrics_bgl, &metrics_shader, "reduce_nodes");
        let metrics_edges_pipeline = make_pipeline("metrics_edges", &metrics_bgl, &metrics_shader, "reduce_edges");

        // ====================================================================
        // Bind groups
        // ====================================================================

        let injection_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("injection_bg"),
            layout: &injection_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_stimulus_groups.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_params.as_entire_binding() },
            ],
        });

        let zone_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zone_bg"),
            layout: &zone_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_zone_state.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_zone_assignments.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_params.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: buf_zone_accum.as_entire_binding() },
            ],
        });

        let source_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("source_bg"),
            layout: &source_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_source_contribs.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_params.as_entire_binding() },
            ],
        });

        let propagation_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("propagation_bg"),
            layout: &propagation_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_source_contribs.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_conductances.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_csr_offsets.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_csr_sources.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: buf_csr_kernels.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: buf_influences.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: buf_params.as_entire_binding() },
            ],
        });

        let dissipate_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dissipate_bg"),
            layout: &dissipate_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_influences.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_params.as_entire_binding() },
            ],
        });

        let plasticity_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("plasticity_bg"),
            layout: &plasticity_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_edges.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_node_delta_dopa.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_params.as_entire_binding() },
            ],
        });

        let budget_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("budget_bg"),
            layout: &budget_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_edges.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_budget_totals.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_params.as_entire_binding() },
            ],
        });

        let sync_cond_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sync_cond_bg"),
            layout: &sync_cond_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_edges.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_csr_edge_indices.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_conductances.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_params.as_entire_binding() },
            ],
        });

        let readout_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("readout_bg"),
            layout: &readout_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_readout_groups.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_readout_scores.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_params.as_entire_binding() },
            ],
        });

        let metrics_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("metrics_bg"),
            layout: &metrics_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_nodes.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_edges.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_metrics_output.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_params.as_entire_binding() },
            ],
        });

        eprintln!(
            "[GPU] GPU-First initialized: {} nodes, {} edges, {} zones, workgroup_size={}",
            num_nodes, num_edges, num_zones, workgroup_size
        );

        Some(GpuContext {
            device,
            queue,
            reset_pipeline,
            injection_pipeline,
            zone_measure_pipeline,
            zone_pid_pipeline,
            zone_apply_pipeline,
            source_contribs_pipeline,
            propagation_pipeline,
            apply_dissipate_pipeline,
            plasticity_pipeline,
            budget_sum_pipeline,
            budget_scale_pipeline,
            sync_conductances_pipeline,
            readout_pipeline,
            metrics_nodes_pipeline,
            metrics_edges_pipeline,
            buf_nodes,
            buf_edges,
            buf_conductances,
            buf_source_contribs,
            buf_influences,
            buf_csr_offsets,
            buf_csr_sources,
            buf_csr_kernels,
            buf_csr_edge_indices,
            buf_node_delta_dopa,
            buf_params,
            buf_budget_totals,
            buf_stimulus_groups,
            buf_zone_assignments,
            buf_zone_state,
            buf_zone_accum,
            buf_readout_groups,
            buf_readout_scores,
            buf_metrics_output,
            staging_readout,
            staging_edges,
            staging_nodes,
            staging_metrics,
            injection_bg,
            zone_bg,
            source_bg,
            propagation_bg,
            dissipate_bg,
            plasticity_bg,
            budget_bg,
            sync_cond_bg,
            readout_bg,
            metrics_bg,
            timestamp_query_set,
            timestamp_resolve_buf,
            timestamp_staging_buf,
            timestamp_period,
            num_nodes,
            num_edges,
            num_zones,
            num_classes,
            group_size,
            workgroup_size,
        })
    }

    // ========================================================================
    // Upload methods (called once after setup, not per tick)
    // ========================================================================

    /// Upload I/O group indices (stimulus + readout). Called after setup_v4_training().
    pub fn upload_io_groups(&self, stimulus_groups: &[Vec<usize>], readout_groups: &[Vec<usize>]) {
        let flat_stim: Vec<u32> = stimulus_groups.iter()
            .flat_map(|g| g.iter().map(|&idx| idx as u32))
            .collect();
        if !flat_stim.is_empty() {
            self.queue.write_buffer(&self.buf_stimulus_groups, 0, bytemuck::cast_slice(&flat_stim));
        }

        let flat_readout: Vec<u32> = readout_groups.iter()
            .flat_map(|g| g.iter().map(|&idx| idx as u32))
            .collect();
        if !flat_readout.is_empty() {
            self.queue.write_buffer(&self.buf_readout_groups, 0, bytemuck::cast_slice(&flat_readout));
        }
    }

    /// Upload spatial dopamine modulation per node (called when reward center changes).
    pub fn upload_node_delta_dopa(&self, delta_dopa: &[f32]) {
        self.queue.write_buffer(&self.buf_node_delta_dopa, 0, bytemuck::cast_slice(delta_dopa));
    }

    // ========================================================================
    // GPU-First tick execution
    // ========================================================================

    /// Execute a complete tick on the GPU. Single CommandEncoder, single submit,
    /// zero intermediate readback. Only syncs if readout is needed.
    pub fn run_full_tick(&self, params: &GpuParams, need_readout: bool, reset_activations: bool) {
        // Upload params uniform
        self.queue.write_buffer(&self.buf_params, 0, bytemuck::bytes_of(params));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("full_tick"),
        });

        // Clear accumulators
        encoder.clear_buffer(&self.buf_zone_accum, 0, None);
        encoder.clear_buffer(&self.buf_budget_totals, 0, None);
        if need_readout {
            encoder.clear_buffer(&self.buf_readout_scores, 0, None);
        }

        let dispatch_nodes = Self::dispatch_2d((self.num_nodes + self.workgroup_size - 1) / self.workgroup_size);
        let dispatch_edges = Self::dispatch_2d((self.num_edges + self.workgroup_size - 1) / self.workgroup_size);

        // Phase 0: Reset activations (at trial start)
        if reset_activations {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.reset_pipeline);
            pass.set_bind_group(0, &self.injection_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Phase 1: Stimulus injection
        if params.stimulus_class >= 0 {
            let stim_dispatch = (self.num_classes * self.group_size + 63) / 64;
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.injection_pipeline);
            pass.set_bind_group(0, &self.injection_bg, &[]);
            pass.dispatch_workgroups(stim_dispatch, 1, 1);
        }

        // Phase 2: Zone measure (accumulate activations per zone)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.zone_measure_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Phase 3: Zone PID (1 thread per zone)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.zone_pid_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }

        // Phase 4: Zone apply (theta_mod + gain_mod → nodes)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.zone_apply_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Phase 5: Source contributions
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.source_contribs_pipeline);
            pass.set_bind_group(0, &self.source_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Phase 6: Propagation (CSR)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.propagation_pipeline);
            pass.set_bind_group(0, &self.propagation_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Phase 7: Apply influences + dissipation (fused)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.apply_dissipate_pipeline);
            pass.set_bind_group(0, &self.dissipate_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Phase 8: Plasticity (STDP + 3-factor + homeostasis + consolidation)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.plasticity_pipeline);
            pass.set_bind_group(0, &self.plasticity_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }

        // Phase 9: Budget normalization (sum + scale)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.budget_sum_pipeline);
            pass.set_bind_group(0, &self.budget_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.budget_scale_pipeline);
            pass.set_bind_group(0, &self.budget_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }

        // Phase 10: Sync conductances (edge → CSR order, GPU-to-GPU)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.sync_conductances_pipeline);
            pass.set_bind_group(0, &self.sync_cond_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }

        // Phase 11: Readout (only if needed)
        if need_readout {
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                pass.set_pipeline(&self.readout_pipeline);
                pass.set_bind_group(0, &self.readout_bg, &[]);
                let readout_dispatch = (self.num_classes * self.group_size + 63) / 64;
                pass.dispatch_workgroups(readout_dispatch, 1, 1);
            }
            encoder.copy_buffer_to_buffer(
                &self.buf_readout_scores, 0,
                &self.staging_readout, 0,
                (self.num_classes as u64) * 4,
            );
        }

        // === SINGLE SUBMIT ===
        self.queue.submit(std::iter::once(encoder.finish()));

        // === SINGLE SYNC (only if readout needed) ===
        if need_readout {
            self.device.poll(wgpu::Maintain::Wait);
        }
    }

    /// Phase names for profiling output (12 phases).
    const PHASE_NAMES: [&str; 12] = [
        "reset", "inject", "zone_measure", "zone_pid", "zone_apply",
        "source_contribs", "propagation", "apply_dissipate", "plasticity",
        "budget_sum", "budget_scale+sync_cond", "readout",
    ];

    /// Run a profiled tick with GPU timestamp queries on each compute pass.
    /// Returns phase timings in microseconds if timestamps are available.
    pub fn run_profiled_tick(&self, params: &GpuParams) -> Option<Vec<(String, f64)>> {
        let qs = self.timestamp_query_set.as_ref()?;
        let resolve_buf = self.timestamp_resolve_buf.as_ref()?;
        let staging_buf = self.timestamp_staging_buf.as_ref()?;

        self.queue.write_buffer(&self.buf_params, 0, bytemuck::bytes_of(params));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("profiled_tick"),
        });

        encoder.clear_buffer(&self.buf_zone_accum, 0, None);
        encoder.clear_buffer(&self.buf_budget_totals, 0, None);
        encoder.clear_buffer(&self.buf_readout_scores, 0, None);

        let dispatch_nodes = Self::dispatch_2d((self.num_nodes + self.workgroup_size - 1) / self.workgroup_size);
        let dispatch_edges = Self::dispatch_2d((self.num_edges + self.workgroup_size - 1) / self.workgroup_size);

        macro_rules! ts_pass {
            ($encoder:expr, $label:expr, $idx:expr) => {
                wgpu::ComputePassDescriptor {
                    label: Some($label),
                    timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                        query_set: qs,
                        beginning_of_pass_write_index: Some($idx * 2),
                        end_of_pass_write_index: Some($idx * 2 + 1),
                    }),
                }
            };
        }

        // Phase 0: Reset
        {
            let desc = ts_pass!(encoder, "reset", 0);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.reset_pipeline);
            pass.set_bind_group(0, &self.injection_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }
        // Phase 1: Inject
        {
            let desc = ts_pass!(encoder, "inject", 1);
            let stim_dispatch = (self.num_classes * self.group_size + 63) / 64;
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.injection_pipeline);
            pass.set_bind_group(0, &self.injection_bg, &[]);
            pass.dispatch_workgroups(stim_dispatch, 1, 1);
        }
        // Phase 2: Zone measure
        {
            let desc = ts_pass!(encoder, "zone_measure", 2);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.zone_measure_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }
        // Phase 3: Zone PID
        {
            let desc = ts_pass!(encoder, "zone_pid", 3);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.zone_pid_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        // Phase 4: Zone apply
        {
            let desc = ts_pass!(encoder, "zone_apply", 4);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.zone_apply_pipeline);
            pass.set_bind_group(0, &self.zone_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }
        // Phase 5: Source contribs
        {
            let desc = ts_pass!(encoder, "source_contribs", 5);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.source_contribs_pipeline);
            pass.set_bind_group(0, &self.source_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }
        // Phase 6: Propagation
        {
            let desc = ts_pass!(encoder, "propagation", 6);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.propagation_pipeline);
            pass.set_bind_group(0, &self.propagation_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }
        // Phase 7: Apply+dissipate
        {
            let desc = ts_pass!(encoder, "apply_dissipate", 7);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.apply_dissipate_pipeline);
            pass.set_bind_group(0, &self.dissipate_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }
        // Phase 8: Plasticity
        {
            let desc = ts_pass!(encoder, "plasticity", 8);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.plasticity_pipeline);
            pass.set_bind_group(0, &self.plasticity_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }
        // Phase 9: Budget sum
        {
            let desc = ts_pass!(encoder, "budget_sum", 9);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.budget_sum_pipeline);
            pass.set_bind_group(0, &self.budget_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }
        // Phase 10: Budget scale + sync conductances
        {
            let desc = ts_pass!(encoder, "budget_scale+sync_cond", 10);
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.budget_scale_pipeline);
            pass.set_bind_group(0, &self.budget_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.sync_conductances_pipeline);
            pass.set_bind_group(0, &self.sync_cond_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }
        // Phase 11: Readout
        {
            let desc = ts_pass!(encoder, "readout", 11);
            let readout_dispatch = (self.num_classes * self.group_size + 63) / 64;
            let mut pass = encoder.begin_compute_pass(&desc);
            pass.set_pipeline(&self.readout_pipeline);
            pass.set_bind_group(0, &self.readout_bg, &[]);
            pass.dispatch_workgroups(readout_dispatch, 1, 1);
        }

        encoder.copy_buffer_to_buffer(
            &self.buf_readout_scores, 0,
            &self.staging_readout, 0,
            (self.num_classes as u64) * 4,
        );

        // Resolve timestamps
        let ts_count = 24u32;
        encoder.resolve_query_set(qs, 0..ts_count, resolve_buf, 0);
        encoder.copy_buffer_to_buffer(resolve_buf, 0, staging_buf, 0, (ts_count as u64) * 8);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);

        // Read timestamps
        let slice = staging_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let timestamps: &[u64] = bytemuck::cast_slice(&data);

        let period_ns = self.timestamp_period as f64;
        let mut timings = Vec::with_capacity(12);
        for i in 0..12usize {
            let begin = timestamps[i * 2];
            let end = timestamps[i * 2 + 1];
            let us = (end.wrapping_sub(begin)) as f64 * period_ns / 1000.0;
            timings.push((Self::PHASE_NAMES[i].to_string(), us));
        }

        drop(data);
        staging_buf.unmap();

        Some(timings)
    }

    /// Check if GPU timestamp profiling is available.
    pub fn has_timestamp_profiling(&self) -> bool {
        self.timestamp_query_set.is_some()
    }

    /// Read readout scores after run_full_tick with need_readout=true.
    /// Returns activation sums per class (decoded from fixed-point).
    pub fn read_readout_scores(&self, num_classes: usize) -> Vec<f32> {
        let slice = self.staging_readout.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let fixed: &[u32] = bytemuck::cast_slice(&data);
        let result: Vec<f32> = fixed.iter()
            .take(num_classes)
            .map(|&v| v as f32 / 1_000_000.0)
            .collect();
        drop(data);
        self.staging_readout.unmap();
        result
    }

    /// Run GPU-side metrics reduction and read back the result.
    /// This is much cheaper than sync_state_to_domain since it only transfers 64 bytes.
    /// Assumes params were already uploaded by run_full_tick in the same tick.
    pub fn compute_gpu_metrics(&self) -> GpuMetrics {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("metrics_reduce"),
        });

        // Clear the metrics output buffer (all atomics to 0)
        encoder.clear_buffer(&self.buf_metrics_output, 0, None);

        let dispatch_nodes = Self::dispatch_2d((self.num_nodes + 255) / 256);
        let dispatch_edges = Self::dispatch_2d((self.num_edges + 255) / 256);

        // Reduce nodes
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.metrics_nodes_pipeline);
            pass.set_bind_group(0, &self.metrics_bg, &[]);
            pass.dispatch_workgroups(dispatch_nodes.0, dispatch_nodes.1, 1);
        }

        // Reduce edges
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.metrics_edges_pipeline);
            pass.set_bind_group(0, &self.metrics_bg, &[]);
            pass.dispatch_workgroups(dispatch_edges.0, dispatch_edges.1, 1);
        }

        // Copy to staging
        let metrics_size = std::mem::size_of::<GpuMetricsRaw>() as u64;
        encoder.copy_buffer_to_buffer(
            &self.buf_metrics_output, 0,
            &self.staging_metrics, 0,
            metrics_size,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);

        // Map and read
        let slice = self.staging_metrics.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let raw: GpuMetricsRaw = *bytemuck::from_bytes(&data[..std::mem::size_of::<GpuMetricsRaw>()]);
        drop(data);
        self.staging_metrics.unmap();

        // Decode fixed-point values
        let nn = self.num_nodes.max(1) as f32;
        let ne = self.num_edges.max(1) as f32;
        GpuMetrics {
            active_count: raw.active_count,
            global_energy: raw.sum_energy_fixed as f32 / 1000.0,
            max_activation: f32::from_bits(raw.max_activation_bits),
            mean_memory_trace: (raw.sum_trace_fixed as f32 / 1000.0) / nn,
            max_memory_trace: f32::from_bits(raw.max_trace_bits),
            mean_fatigue: (raw.sum_fatigue_fixed as f32 / 1000.0) / nn,
            active_excitatory: raw.active_excitatory,
            active_inhibitory: raw.active_inhibitory,
            excitatory_energy: raw.sum_exc_energy_fixed as f32 / 1000.0,
            inhibitory_energy: raw.sum_inh_energy_fixed as f32 / 1000.0,
            mean_conductance: (raw.sum_conductance_fixed as f32 / 1000.0) / ne,
            max_conductance: f32::from_bits(raw.max_conductance_bits),
            consolidated_count: raw.consolidated_count,
            mean_eligibility: (raw.sum_eligibility_fixed as f32 / 1_000_000.0) / ne,
        }
    }

    // ========================================================================
    // Periodic downloads (for metrics / viz / final summary)
    // ========================================================================

    /// Download current node state from GPU to a Vec<GpuNode>.
    /// Used for metrics snapshots and visualization.
    pub fn download_nodes(&self) -> Vec<GpuNode> {
        let size = (self.num_nodes as u64) * std::mem::size_of::<GpuNode>() as u64;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("download_nodes"),
        });
        encoder.copy_buffer_to_buffer(&self.buf_nodes, 0, &self.staging_nodes, 0, size);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = self.staging_nodes.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let nodes: Vec<GpuNode> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        self.staging_nodes.unmap();
        nodes
    }

    /// Sync GPU node/edge state back to CPU Domain (for metrics/final summary).
    pub fn sync_state_to_domain(&self, domain: &mut Domain) {
        // Download nodes
        let gpu_nodes = self.download_nodes();
        for (i, gn) in gpu_nodes.iter().enumerate() {
            domain.nodes[i].activation = gn.activation;
            domain.nodes[i].threshold = gn.threshold;
            domain.nodes[i].fatigue = gn.fatigue;
            domain.nodes[i].memory_trace = gn.memory_trace;
            domain.nodes[i].excitability = gn.excitability;
            domain.nodes[i].inhibition = gn.inhibition;
            domain.nodes[i].threshold_mod = gn.threshold_mod;
            domain.nodes[i].last_activation_tick = if gn.last_activation_tick >= 0 {
                Some(gn.last_activation_tick as usize)
            } else {
                None
            };
            domain.nodes[i].activation_count = gn.activation_count as u64;
        }

        // Download edges
        let edge_size = (self.num_edges as u64) * std::mem::size_of::<GpuEdge>() as u64;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("download_edges"),
        });
        encoder.copy_buffer_to_buffer(&self.buf_edges, 0, &self.staging_edges, 0, edge_size);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = self.staging_edges.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let gpu_edges: &[GpuEdge] = bytemuck::cast_slice(&data);

        // If edges were compacted (dropped for GPU mode), rebuild from GPU data
        if domain.edges.is_empty() && !gpu_edges.is_empty() {
            use crate::edge::Edge;
            domain.edges = gpu_edges.iter().map(|ge| {
                let mut e = Edge::default_with_endpoints(ge.from as usize, ge.to as usize);
                e.conductance = ge.conductance;
                e.distance = ge.distance;
                e.eligibility = ge.eligibility;
                e.consolidated = ge.consolidated != 0;
                e.consolidation_counter = ge.consolidation_counter as usize;
                e
            }).collect();
        } else {
            for (i, ge) in gpu_edges.iter().enumerate() {
                domain.edges[i].conductance = ge.conductance;
                domain.edges[i].eligibility = ge.eligibility;
                domain.edges[i].consolidated = ge.consolidated != 0;
                domain.edges[i].consolidation_counter = ge.consolidation_counter as usize;
            }
        }

        drop(data);
        self.staging_edges.unmap();
        domain.sync_conductances();
    }
}

/// Helper to create a BindGroupLayout entry.
fn bgl_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
