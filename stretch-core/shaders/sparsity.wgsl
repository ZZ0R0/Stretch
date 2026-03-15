// sparsity.wgsl — V6: Wavefront-aware sparsity enforcement
// 1 thread per node. Suppresses neurons whose novelty-weighted score
// is below the threshold (computed on CPU and passed via GpuParams).

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
    // V6 fields
    sparsity_enabled: u32,
    max_active_count: u32,
    suppress_factor: f32,
    novelty_gain: f32,
    novelty_window: u32,
    dopa_mod_enabled: u32,
    reverb_min: f32,
    reverb_max: f32,
    decay_mod_strength: f32,
    dopa_threshold: f32,
    dopa_kappa: f32,
    _pad_v6_0: u32,
    _pad_v6_1: u32,
    _pad_v6_2: u32,
};

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read_write> first_activation_tick: array<u32>;
@group(0) @binding(2) var<uniform>             params: GpuParams;
@group(0) @binding(3) var<uniform>             sparsity_threshold: f32;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }
    if (params.sparsity_enabled == 0u) { return; }

    var node = nodes[idx];
    let act = node.activation;

    // Skip near-zero activations
    if (act < 0.01) { return; }

    // Update first_activation_tick if node just became active
    let eff_threshold = max(
        (node.threshold + node.fatigue + node.inhibition + node.threshold_mod)
        / max(node.excitability, 0.01),
        0.05
    );
    if (act > eff_threshold && first_activation_tick[idx] == 0xFFFFFFFFu) {
        first_activation_tick[idx] = params.current_tick;
    }

    // Compute novelty bonus
    let first_tick = first_activation_tick[idx];
    var bonus = 1.0;
    if (first_tick < 0xFFFFFFFFu) {
        let age = f32(params.current_tick - first_tick);
        let window = f32(params.novelty_window);
        let raw = max(0.0, window - age) / max(window, 1.0);
        bonus = 1.0 + params.novelty_gain * raw;
    } else {
        // Never activated yet → full novelty bonus
        bonus = 1.0 + params.novelty_gain;
    }

    let score = act * bonus;

    // Suppress if below threshold
    if (score < sparsity_threshold) {
        node.activation *= params.suppress_factor;
        nodes[idx] = node;
    }
}
