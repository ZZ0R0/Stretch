// adaptive_decay.wgsl — V5.2: adaptive activation decay based on local activity
// alpha_eff_i = alpha_base * clamp(1 - k_local * mean_neighbor_activity, 0.2, 1.0)
// Uses outgoing CSR to compute local neighborhood activity

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
@group(0) @binding(1) var<storage, read>       out_offsets: array<u32>;
@group(0) @binding(2) var<storage, read>       out_targets: array<u32>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }
    if (params.adaptive_decay_enabled == 0u) { return; }

    let act = nodes[idx].activation;
    if (act <= params.activation_min) { return; }

    // Compute mean neighbor activity via outgoing CSR
    let start = out_offsets[idx];
    let end = out_offsets[idx + 1u];
    var sum: f32 = 0.0;
    let count = end - start;
    if (count == 0u) {
        // No neighbors: use base decay
        let new_act = act * (1.0 - params.activation_decay);
        nodes[idx].activation = max(new_act, params.activation_min);
        return;
    }

    for (var k = start; k < end; k++) {
        sum += nodes[out_targets[k]].activation;
    }
    let local_act = sum / f32(count);
    let factor = clamp(1.0 - params.k_local * local_act, 0.2, 1.0);
    let effective_decay = params.activation_decay * factor;
    let new_act = act * (1.0 - effective_decay);
    nodes[idx].activation = max(new_act, params.activation_min);
}
