// apply_and_dissipate.wgsl — apply influences + full dissipation (fused)
// 1 thread per node. Replaces apply_influences() + dissipation CPU loop.

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
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<storage, read_write> nodes: array<GpuNode>;
@group(0) @binding(1) var<storage, read>       influences: array<f32>;
@group(0) @binding(2) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_nodes) { return; }

    var node = nodes[idx];

    // --- 1. Apply influence ---
    let infl = influences[idx];
    node.activation = clamp(node.activation + infl, 0.0, 10.0);

    // --- 2. Check if active (for STDP tick update + dissipation gains) ---
    let eff_threshold = max(
        (node.threshold + node.fatigue + node.inhibition + node.threshold_mod)
        / max(node.excitability, 0.01),
        0.05
    );
    let was_active = node.activation > eff_threshold;

    if (was_active) {
        node.last_activation_tick = i32(params.current_tick);
        node.activation_count += 1u;
    }

    // --- 3. Dissipation ---

    // Decay jitter: simple hash for reproducible per-node jitter
    var effective_decay = params.activation_decay;
    if (params.decay_jitter > 0.0) {
        let h = (idx * 2654435769u + params.current_tick * 2246822519u);
        let jitter_val = (f32(h >> 16u) / 65536.0 - 0.5) * 2.0 * params.decay_jitter;
        effective_decay = clamp(effective_decay * (1.0 + jitter_val), 0.0, 1.0);
    }

    // Fatigue update
    if (was_active) {
        node.fatigue += params.fatigue_gain * node.activation;
    }
    node.fatigue = clamp(node.fatigue * (1.0 - params.fatigue_recovery), 0.0, 10.0);

    // Inhibition update
    if (was_active) {
        node.inhibition += params.inhibition_gain;
    }
    node.inhibition = clamp(node.inhibition * (1.0 - params.inhibition_decay), 0.0, 10.0);

    // Memory trace update
    if (was_active) {
        node.memory_trace += params.trace_gain * node.activation;
    }
    node.memory_trace = clamp(node.memory_trace * (1.0 - params.trace_decay), 0.0, 100.0);

    // Excitability from trace
    node.excitability = 1.0 + 0.1 * min(node.memory_trace, 5.0);

    // Activation decay
    node.activation *= (1.0 - effective_decay);
    node.activation = max(node.activation, params.activation_min);

    // --- 4. Write back ---
    nodes[idx] = node;
}
