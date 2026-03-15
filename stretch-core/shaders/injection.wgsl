// injection.wgsl — inject stimulus into input group nodes
// 1 thread per node in the stimulus group (small dispatch ~50-100 threads)

struct GpuNode {
    activation: f32,
    threshold: f32,
    fatigue: f32,
    memory_trace: f32,
    excitability: f32,
    inhibition: f32,
    threshold_mod: f32,
    last_activation_tick: i32,
    activation_count: u32,
    is_excitatory: u32,
    gain_mod: f32,
    _pad: f32,
};

struct GpuParams {
    num_nodes: u32,
    num_edges: u32,
    current_tick: u32,
    propagation_gain: f32,
    gain_inhibitory: f32,
    activation_decay: f32,
    activation_min: f32,
    fatigue_gain: f32,
    fatigue_recovery: f32,
    inhibition_gain: f32,
    inhibition_decay: f32,
    trace_gain: f32,
    trace_decay: f32,
    decay_jitter: f32,
    a_plus: f32,
    a_minus: f32,
    tau_plus: f32,
    tau_minus: f32,
    elig_decay: f32,
    elig_max: f32,
    plasticity_gain: f32,
    global_delta_dopa: f32,
    dopa_phasic: f32,
    use_spatial: u32,
    spatial_lambda: f32,
    cond_min: f32,
    cond_max: f32,
    homeostatic_rate: f32,
    baseline_cond: f32,
    dopamine_level: f32,
    dopa_consol_threshold: f32,
    consol_conductance_threshold: f32,
    consol_ticks_required: u32,
    budget: f32,
    stimulus_class: i32,
    stimulus_intensity: f32,
    num_zones: u32,
    zone_kp: f32,
    zone_ki: f32,
    zone_kd: f32,
    zone_pid_output_max: f32,
    zone_pid_integral_max: f32,
    zone_k_theta: f32,
    zone_k_gain: f32,
    stimulus_group_size: u32,
    reset_policy: u32,
    adaptive_decay_enabled: u32,
    k_local: f32,
    reverberation_enabled: u32,
    reverb_gain: f32,
    rpe_delta: f32,
    rho_boost: f32,
    plasticity_disabled: u32,
    num_classes: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read>       stimulus_groups: array<u32>;
@group(0) @binding(2) var<uniform>             params: GpuParams;

// Reset activations based on V5.2 reset_policy (dispatched at trial start)
// 0 = full (all to 0), 1 = partial (keep 10%), 2 = none
@compute @workgroup_size(256)
fn reset_activations(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }
    let policy = params.reset_policy;
    if (policy == 0u) {
        nodes[idx].activation = 0.0;
    } else if (policy == 1u) {
        nodes[idx].activation *= 0.1;
    }
    // policy == 2u (none): do nothing
}

// Inject stimulus into input group nodes
@compute @workgroup_size(64)
fn inject(@builtin(global_invocation_id) gid: vec3<u32>) {
    let stim_class = params.stimulus_class;
    if (stim_class < 0) { return; }

    let group_size = params.stimulus_group_size;
    let base = u32(stim_class) * group_size;
    let k = gid.x;
    if (k >= group_size) { return; }

    let node_idx = stimulus_groups[base + k];
    nodes[node_idx].activation = min(nodes[node_idx].activation + params.stimulus_intensity, 10.0);
}
