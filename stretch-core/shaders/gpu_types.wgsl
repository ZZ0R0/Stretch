// gpu_types.wgsl — shared struct definitions for all GPU-first shaders
// These are included (copy-pasted) into each shader since WGSL has no #include.

// NOTE: This file is a REFERENCE. Each shader duplicates these definitions.
// Keep them in sync when modifying.

struct GpuNode {
    activation: f32,            // offset 0
    threshold: f32,             // offset 4
    fatigue: f32,               // offset 8
    memory_trace: f32,          // offset 12
    excitability: f32,          // offset 16
    inhibition: f32,            // offset 20
    threshold_mod: f32,         // offset 24
    last_activation_tick: i32,  // offset 28
    activation_count: u32,      // offset 32
    is_excitatory: u32,         // offset 36
    gain_mod: f32,              // offset 40
    _pad: f32,                  // offset 44 → 48 bytes total
};

struct GpuParams {
    // --- Sizes ---
    num_nodes: u32,             // 0
    num_edges: u32,             // 4
    current_tick: u32,          // 8
    // --- Propagation ---
    propagation_gain: f32,      // 12
    gain_inhibitory: f32,       // 16
    // --- Dissipation ---
    activation_decay: f32,      // 20
    activation_min: f32,        // 24
    fatigue_gain: f32,          // 28
    fatigue_recovery: f32,      // 32
    inhibition_gain: f32,       // 36
    inhibition_decay: f32,      // 40
    trace_gain: f32,            // 44
    trace_decay: f32,           // 48
    decay_jitter: f32,          // 52
    // --- STDP / Plasticity ---
    a_plus: f32,                // 56
    a_minus: f32,               // 60
    tau_plus: f32,              // 64
    tau_minus: f32,             // 68
    elig_decay: f32,            // 72
    elig_max: f32,              // 76
    plasticity_gain: f32,       // 80
    global_delta_dopa: f32,     // 84
    dopa_phasic: f32,           // 88
    use_spatial: u32,           // 92
    spatial_lambda: f32,        // 96
    cond_min: f32,              // 100
    cond_max: f32,              // 104
    homeostatic_rate: f32,      // 108
    baseline_cond: f32,         // 112
    dopamine_level: f32,        // 116
    dopa_consol_threshold: f32, // 120
    consol_conductance_threshold: f32, // 124
    consol_ticks_required: u32, // 128
    budget: f32,                // 132
    // --- Stimulus ---
    stimulus_class: i32,        // 136
    stimulus_intensity: f32,    // 140
    // --- Zone PID ---
    num_zones: u32,             // 144
    zone_kp: f32,               // 148
    zone_ki: f32,               // 152
    zone_kd: f32,               // 156
    zone_pid_output_max: f32,   // 160
    zone_pid_integral_max: f32, // 164
    zone_k_theta: f32,          // 168
    zone_k_gain: f32,           // 172
    // --- Stimulus groups ---
    stimulus_group_size: u32,   // 176
    // --- Padding to 16-byte alignment ---
    _pad0: u32,                 // 180
    _pad1: u32,                 // 184
    _pad2: u32,                 // 188
    // Total: 192 bytes (divisible by 16)
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
    // 48 bytes
};
