// metrics_reduce.wgsl — parallel reduce for aggregated metrics
// Two entry points:
//   reduce_nodes: dispatched over nodes, aggregates node metrics
//   reduce_edges: dispatched over edges, aggregates edge metrics
// Output: small MetricsOutput buffer (~64 bytes) using atomics.
// Sums use fixed-point (×1000). Max uses bitcast trick for positive floats.

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

struct GpuEdge {
    from_node: u32,
    to_node: u32,
    conductance: f32,
    eligibility: f32,
    consolidated: u32,
    consolidation_counter: u32,
    distance: f32,
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

// Output layout: 16 × u32 = 64 bytes
// Indices:
//  0: active_count
//  1: sum_energy_fixed       (activation × 1000)
//  2: max_activation_bits    (bitcast of f32)
//  3: sum_trace_fixed        (memory_trace × 1000)
//  4: max_trace_bits
//  5: sum_fatigue_fixed      (fatigue × 1000)
//  6: active_excitatory
//  7: active_inhibitory
//  8: sum_exc_energy_fixed
//  9: sum_inh_energy_fixed
// 10: sum_conductance_fixed  (conductance × 1000)
// 11: max_conductance_bits
// 12: consolidated_count
// 13: sum_eligibility_fixed  (|eligibility| × 1000000)
// 14: _pad
// 15: _pad

@group(0) @binding(0) var<storage, read>       nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read>       edges: array<GpuEdge>;
@group(0) @binding(2) var<storage, read_write> metrics: array<atomic<u32>, 16>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn reduce_nodes(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }

    let node = nodes[idx];
    let eff_threshold = (node.threshold + node.threshold_mod) * node.excitability;
    let is_active = node.activation > eff_threshold;

    // Active count
    if (is_active) {
        atomicAdd(&metrics[0], 1u);
    }

    // Sum energy (fixed-point ×1000)
    let energy_fixed = u32(clamp(node.activation, 0.0, 4000000.0) * 1000.0);
    atomicAdd(&metrics[1], energy_fixed);

    // Max activation (bitcast trick: for positive f32, u32 ordering is preserved)
    let act_bits = bitcast<u32>(max(node.activation, 0.0));
    atomicMax(&metrics[2], act_bits);

    // Sum memory_trace (fixed-point ×1000)
    let trace_fixed = u32(clamp(node.memory_trace, 0.0, 1000000.0) * 1000.0);
    atomicAdd(&metrics[3], trace_fixed);

    // Max memory_trace
    let trace_bits = bitcast<u32>(max(node.memory_trace, 0.0));
    atomicMax(&metrics[4], trace_bits);

    // Sum fatigue (fixed-point ×1000)
    let fatigue_fixed = u32(clamp(node.fatigue, 0.0, 1000000.0) * 1000.0);
    atomicAdd(&metrics[5], fatigue_fixed);

    // E/I active counts and energy
    if (node.is_excitatory == 1u) {
        if (is_active) { atomicAdd(&metrics[6], 1u); }
        atomicAdd(&metrics[8], energy_fixed);
    } else {
        if (is_active) { atomicAdd(&metrics[7], 1u); }
        atomicAdd(&metrics[9], energy_fixed);
    }
}

@compute @workgroup_size(256)
fn reduce_edges(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_edges) { return; }

    let edge = edges[idx];

    // Sum conductance (fixed-point ×1000)
    let cond_fixed = u32(clamp(edge.conductance, 0.0, 1000000.0) * 1000.0);
    atomicAdd(&metrics[10], cond_fixed);

    // Max conductance
    let cond_bits = bitcast<u32>(max(edge.conductance, 0.0));
    atomicMax(&metrics[11], cond_bits);

    // Consolidated count
    if (edge.consolidated != 0u) {
        atomicAdd(&metrics[12], 1u);
    }

    // Sum |eligibility| (fixed-point ×1000000 for precision)
    let elig_fixed = u32(clamp(abs(edge.eligibility), 0.0, 4000.0) * 1000000.0);
    atomicAdd(&metrics[13], elig_fixed);
}
