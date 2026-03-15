// sync_conductances.wgsl — GPU-to-GPU conductance reorder
// 1 thread per CSR entry.
// Copies edges[csr_edge_indices[k]].conductance → conductances[k]
// Replaces the CPU round-trip: download edges → reorder → upload conductances.

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

@group(0) @binding(0) var<storage, read>       edges: array<GpuEdge>;
@group(0) @binding(1) var<storage, read>       csr_edge_indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> conductances: array<f32>;
@group(0) @binding(3) var<uniform>             params: GpuParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>) {
    let k = gid.y * (nwg.x * 256u) + gid.x;
    if (k >= params.num_edges) { return; }

    let edge_idx = csr_edge_indices[k];
    conductances[k] = edges[edge_idx].conductance;
}
