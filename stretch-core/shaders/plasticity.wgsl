// plasticity.wgsl — one thread per edge (577k edges)
// Implements: STDP ψ, eligibility, three-factor, homeostatic decay, consolidation
// V4.3: Reads last_activation_tick from buf_nodes instead of separate buffer

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

@group(0) @binding(0) var<storage, read_write> edges: array<GpuEdge>;
@group(0) @binding(1) var<storage, read>       nodes: array<GpuNode>;
@group(0) @binding(2) var<storage, read>       node_delta_dopa: array<f32>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let idx = gid.y * (nwg.x * 256u) + gid.x;
    if (idx >= params.num_edges) {
        return;
    }

    // V5.2: skip all plasticity when disabled (TopologyOnly / RandomBaseline)
    if (params.plasticity_disabled != 0u) {
        return;
    }

    let src = edges[idx].from_node;
    let dst = edges[idx].to_node;

    // --- Step 1: STDP ψ ---
    let t_pre = nodes[src].last_activation_tick;
    let t_post = nodes[dst].last_activation_tick;
    var psi: f32 = 0.0;

    if (t_pre >= 0 && t_post >= 0 && t_pre != t_post) {
        let dt = f32(t_post - t_pre);
        if (dt > 0.0) {
            psi = params.a_plus * exp(-dt / params.tau_plus);
        } else {
            psi = -params.a_minus * exp(dt / params.tau_minus);
        }
    }

    // --- Step 2: Eligibility trace update ---
    var elig = params.elig_decay * edges[idx].eligibility + psi;
    elig = clamp(elig, -params.elig_max, params.elig_max);
    edges[idx].eligibility = elig;

    // Skip steps 3-5 if eligibility is negligible (GPU equivalent of hot edges)
    if (abs(elig) < 1e-6) {
        return;
    }

    // --- Step 3: Three-factor learning ---
    var delta_d: f32;
    if (params.use_spatial != 0u) {
        delta_d = params.dopa_phasic * node_delta_dopa[dst];
    } else {
        delta_d = params.global_delta_dopa;
    }

    let dw = params.plasticity_gain * delta_d * elig;
    var cond = edges[idx].conductance + dw;
    cond = clamp(cond, params.cond_min, params.cond_max);
    edges[idx].conductance = cond;

    // --- Step 4: Homeostatic decay (skip if consolidated) ---
    // V5.2: accelerated forgetting — ρ_eff = ρ₀ + ρ_boost × max(0, -δ)
    if (edges[idx].consolidated == 0u) {
        cond = edges[idx].conductance;
        var rho_eff = params.homeostatic_rate;
        if (params.rho_boost > 0.0) {
            rho_eff = rho_eff + params.rho_boost * max(0.0, -params.rpe_delta);
        }
        cond = cond + rho_eff * (params.baseline_cond - cond);
        cond = clamp(cond, params.cond_min, params.cond_max);
        edges[idx].conductance = cond;
    }

    // --- Step 5: Consolidation ---
    if (params.dopamine_level > params.dopa_consol_threshold && elig > 0.0) {
        let counter = edges[idx].consolidation_counter + 1u;
        edges[idx].consolidation_counter = counter;
        if (counter >= params.consol_ticks_required) {
            edges[idx].consolidated = 1u;
        }
    }
}
