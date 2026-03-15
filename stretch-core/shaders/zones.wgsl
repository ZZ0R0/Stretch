// zones.wgsl — zone PID regulation on GPU
// 3 entry points: zone_measure, zone_pid, zone_apply

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

struct GpuZone {
    target_activity: f32,
    activity_sum: f32,
    member_count: u32,
    error: f32,
    integral: f32,
    error_prev: f32,
    output: f32,
    theta_mod: f32,
    gain_mod: f32,
    stable_ticks: u32,
    is_stable: u32,
    _pad: f32,
};

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read_write> zones: array<GpuZone>;
@group(0) @binding(2) var<storage, read>       zone_assignments: array<u32>;
@group(0) @binding(3) var<uniform>             params: GpuParams;
@group(0) @binding(4) var<storage, read_write> zone_accum: array<atomic<u32>>;

// --- Pass 1: accumulate activations per zone (atomically) ---
// 1 thread per node
@compute @workgroup_size(256)
fn zone_measure(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }

    let zone_id = zone_assignments[idx];
    let activation = nodes[idx].activation;

    // Fixed-point atomic add: activation × 1000000 → u32
    let act_fixed = u32(clamp(activation, 0.0, 4000.0) * 1000000.0);
    atomicAdd(&zone_accum[zone_id], act_fixed);
}

// --- Pass 2: PID per zone (1 thread per zone) ---
@compute @workgroup_size(64)
fn zone_pid(@builtin(global_invocation_id) gid: vec3<u32>) {
    let z = gid.x;
    if (z >= params.num_zones) { return; }

    let sum_fixed = atomicLoad(&zone_accum[z]);
    let count = zones[z].member_count;
    var mean = 0.0;
    if (count > 0u) {
        mean = f32(sum_fixed) / 1000000.0 / f32(count);
    }
    zones[z].activity_sum = mean;

    // C6: skip stable zones except every 10 ticks for reactivation check
    if (zones[z].is_stable != 0u && zones[z].stable_ticks % 10u != 0u) {
        zones[z].stable_ticks += 1u;
        return;
    }

    // PID computation
    let error = zones[z].target_activity - mean;
    let integral = clamp(
        zones[z].integral + error,
        -params.zone_pid_integral_max,
        params.zone_pid_integral_max
    );
    let derivative = error - zones[z].error_prev;
    let u = clamp(
        params.zone_kp * error + params.zone_ki * integral + params.zone_kd * derivative,
        -params.zone_pid_output_max,
        params.zone_pid_output_max
    );

    zones[z].error_prev = error;
    zones[z].error = error;
    zones[z].integral = integral;
    zones[z].output = u;

    // Indirect mode: theta_mod, gain_mod
    zones[z].theta_mod = -params.zone_k_theta * u;
    zones[z].gain_mod = params.zone_k_gain * u;

    // Stability tracking (C6)
    if (abs(error) < 0.01) {
        zones[z].stable_ticks += 1u;
        if (zones[z].stable_ticks >= 50u) {
            zones[z].is_stable = 1u;
        }
    } else {
        zones[z].stable_ticks = 0u;
        zones[z].is_stable = 0u;
    }
}

// --- Pass 3: apply theta_mod and gain_mod to member nodes ---
// 1 thread per node
@compute @workgroup_size(256)
fn zone_apply(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }

    let z = zone_assignments[idx];
    nodes[idx].threshold_mod = zones[z].theta_mod;
    nodes[idx].gain_mod = zones[z].gain_mod;
}
