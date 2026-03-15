use rayon::prelude::*;

use crate::config::{ConsolidationConfig, EdgeDefaults, EligibilityConfig, StdpConfig, SynapticBudgetConfig};
use crate::dopamine::DopamineConfig;
use crate::domain::Domain;

/// V4 correctif : Règle des trois facteurs (Frémaux & Gerstner 2016)
/// avec dopamine spatialisée.
///
///   1. STDP calcule ψ_ij (LTP/LTD) mais NE modifie PAS la conductance.
///   2. ψ alimente la trace d'éligibilité : e_ij = γ × e_ij + ψ_ij.
///   3. Règle des trois facteurs avec dopamine locale :
///      δ_d_local = d_phasic × exp(-λ × dist(target_node, reward_center))
///      ΔC_ij = η × δ_d_local × e_ij
///   4. Décroissance homéostatique : C → C₀ lentement.

/// Epsilon pour le seuil hot edges (C3).
const HOT_EDGE_EPSILON: f32 = 1e-6;

/// Pre-compute spatial dopamine vec (only when reward_center changes).
pub fn precompute_spatial_dopamine(
    positions: &[[f64; 3]],
    center: [f64; 3],
    dopa_phasic: f64,
    spatial_lambda: f64,
    buf: &mut Vec<f32>,
) {
    let n = positions.len();
    buf.resize(n, 0.0);
    buf.par_iter_mut().enumerate().for_each(|(i, out)| {
        let pos = &positions[i];
        let dx = pos[0] - center[0];
        let dy = pos[1] - center[1];
        let dz = pos[2] - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        *out = (dopa_phasic * (-spatial_lambda * dist).exp()) as f32;
    });
}

/// Collect activation ticks snapshot into reusable buffer.
pub fn snapshot_activation_ticks(domain: &Domain, buf: &mut Vec<Option<usize>>) {
    let n = domain.nodes.len();
    buf.resize(n, None);
    for (i, node) in domain.nodes.iter().enumerate() {
        buf[i] = node.last_activation_tick;
    }
}

pub fn update_plasticity_stdp_budget(
    domain: &mut Domain,
    consolidation: &ConsolidationConfig,
    stdp_config: &StdpConfig,
    budget_config: &SynapticBudgetConfig,
    edge_defaults: &EdgeDefaults,
    eligibility_config: &EligibilityConfig,
    dopamine_config: &DopamineConfig,
    dopamine_level: f64,
    reward_center: Option<[f64; 3]>,
    tick: usize,
    // V5.2: RPE-modulated forgetting
    rpe_delta: f32,
    rho_boost: f32,
    // Reusable buffers (managed by caller)
    activation_ticks_buf: &mut Vec<Option<usize>>,
    node_delta_dopa_buf: &mut Vec<f32>,
    cached_dopa_center: &mut Option<[f64; 3]>,
) {
    // Phase A : mise à jour des last_activation_tick
    domain.nodes.par_iter_mut().for_each(|node| {
        if node.is_active() {
            node.last_activation_tick = Some(tick);
        }
    });

    // Phase B : snapshot activation ticks
    snapshot_activation_ticks(domain, activation_ticks_buf);

    // Pre-compute spatial dopamine (only when center changes)
    let spatial_lambda = dopamine_config.spatial_lambda;
    let dopa_tonic = dopamine_config.tonic;
    let dopa_phasic = dopamine_level - dopa_tonic;
    let use_spatial = spatial_lambda > 0.0 && reward_center.is_some();

    if use_spatial {
        let center = reward_center.unwrap();
        let need_recompute = match cached_dopa_center {
            Some(prev) => prev[0] != center[0] || prev[1] != center[1] || prev[2] != center[2],
            None => true,
        };
        if need_recompute || node_delta_dopa_buf.len() != domain.positions.len() {
            precompute_spatial_dopamine(
                &domain.positions,
                center,
                1.0, // normalize: store exp(-λd), multiply by dopa_phasic later
                spatial_lambda,
                node_delta_dopa_buf,
            );
            *cached_dopa_center = Some(center);
        }
    }

    let global_delta_dopa = dopa_phasic as f32;

    let a_plus = stdp_config.a_plus as f32;
    let a_minus = stdp_config.a_minus as f32;
    let tau_plus = stdp_config.tau_plus as f32;
    let tau_minus = stdp_config.tau_minus as f32;
    let tau_window = (stdp_config.tau_plus.max(stdp_config.tau_minus)) as usize;

    let elig_decay = eligibility_config.decay as f32;
    let elig_max = eligibility_config.max as f32;

    let dopa_plasticity_gain = dopamine_config.plasticity_gain as f32;
    let dopa_consolidation_threshold = dopamine_config.consolidation_threshold as f32;

    let consolidation_threshold = consolidation.threshold as f32;
    let consolidation_ticks = consolidation.ticks_required;

    let cond_min = edge_defaults.conductance_min as f32;
    let cond_max = edge_defaults.conductance_max as f32;
    let homeostatic_rate = edge_defaults.decay as f32;
    let baseline_cond = edge_defaults.conductance as f32;

    let activation_ticks = &*activation_ticks_buf;
    let node_delta_dopa = &*node_delta_dopa_buf;

    // --- C2: Build active_edge_indices — edges where at least one endpoint fired recently ---
    let recently_active: Vec<bool> = activation_ticks.iter()
        .map(|opt| match opt {
            Some(t) => tick.saturating_sub(*t) <= tau_window,
            None => false,
        })
        .collect();

    // Collect indices of edges touching recently active nodes using outgoing+incoming
    let mut active_edge_set: Vec<bool> = vec![false; domain.edges.len()];
    for (node_idx, &is_recent) in recently_active.iter().enumerate() {
        if !is_recent { continue; }
        // Outgoing edges from this node
        let out_start = domain.outgoing.offsets[node_idx];
        let out_end = domain.outgoing.offsets[node_idx + 1];
        for k in out_start..out_end {
            active_edge_set[domain.outgoing.edge_indices[k]] = true;
        }
        // Incoming edges to this node
        let in_start = domain.incoming.offsets[node_idx];
        let in_end = domain.incoming.offsets[node_idx + 1];
        for k in in_start..in_end {
            active_edge_set[domain.incoming.edge_indices[k]] = true;
        }
    }

    // --- Phase C steps 1-2: STDP ψ + eligibility on active edges only (C2) ---
    // For non-active edges, just decay eligibility
    // We process all edges for eligibility decay but only compute ψ for active ones
    domain.edges.par_iter_mut().enumerate().for_each(|(idx, edge)| {
        if active_edge_set[idx] {
            // --- 1. STDP : calcul de la direction ψ ---
            let mut psi: f32 = 0.0;
            {
                let pre_tick = activation_ticks[edge.from];
                let post_tick = activation_ticks[edge.to];

                if let (Some(t_pre), Some(t_post)) = (pre_tick, post_tick) {
                    if t_pre != t_post {
                        let dt = t_post as f32 - t_pre as f32;
                        psi = if dt > 0.0 {
                            a_plus * (-dt / tau_plus).exp()
                        } else {
                            -a_minus * (dt / tau_minus).exp()
                        };
                    }
                }
            }
            // --- 2. Eligibility update with ψ ---
            edge.eligibility = (elig_decay * edge.eligibility + psi).clamp(-elig_max, elig_max);
        } else {
            // Just decay eligibility for inactive edges
            edge.eligibility *= elig_decay;
            // Clamp small values to zero to keep hot edges list tight
            if edge.eligibility.abs() < HOT_EDGE_EPSILON * 0.1 {
                edge.eligibility = 0.0;
            }
        }
    });

    // --- C3: Update hot edges list ---
    // Rebuild from scratch (more efficient than incremental for first implementation)
    domain.hot_edge_indices.clear();
    for i in 0..domain.edges.len() {
        let is_hot = domain.edges[i].eligibility.abs() > HOT_EDGE_EPSILON;
        domain.edge_is_hot[i] = is_hot;
        if is_hot {
            domain.hot_edge_indices.push(i);
        }
    }

    // --- Phase C steps 3-5: Three-factor + homeostasis + consolidation on hot edges only (C3) ---
    // Track dirty sources for C4 incremental budget
    let n_nodes = domain.nodes.len();
    let dirty_sources: Vec<std::sync::atomic::AtomicBool> =
        (0..n_nodes).map(|_| std::sync::atomic::AtomicBool::new(false)).collect();

    // Clone hot_edge_indices to avoid borrow conflict with domain.edges
    let hot_indices: Vec<usize> = domain.hot_edge_indices.clone();

    // Process hot edges sequentially for steps 3-5 (hot edges are typically ~5-10% of total)
    for &idx in &hot_indices {
        let edge = &mut domain.edges[idx];
        let old_conductance = edge.conductance;

        // --- 3. Règle des trois facteurs avec dopamine locale ---
        {
            let delta_d: f32 = if use_spatial {
                (dopa_phasic as f32) * node_delta_dopa[edge.to]
            } else {
                global_delta_dopa
            };
            let dw = dopa_plasticity_gain * delta_d * edge.eligibility;
            edge.conductance = (edge.conductance + dw).clamp(cond_min, cond_max);
        }

        // --- 4. Décroissance homéostatique ---
        // V5.2: accelerated forgetting when RPE is negative
        let rho_eff = homeostatic_rate + rho_boost * (-rpe_delta).max(0.0);
        if !edge.consolidated {
            edge.conductance += rho_eff * (baseline_cond - edge.conductance);
            edge.conductance = edge.conductance.clamp(cond_min, cond_max);
        }

        // --- 5. Consolidation ---
        if (dopamine_level as f32) > dopa_consolidation_threshold && edge.eligibility > 0.0 {
            edge.update_consolidation(consolidation_threshold, consolidation_ticks);
        }

        // C4: Track delta for incremental budget
        let delta = edge.conductance - old_conductance;
        if delta.abs() > 1e-10 {
            domain.running_totals[edge.from] += delta;
            dirty_sources[edge.from].store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    // --- C4: Phase D — incremental budget normalization ---
    let budget = budget_config.budget as f32;
    let dirty_source_indices: Vec<usize> = dirty_sources.iter()
        .enumerate()
        .filter(|(_, d)| d.load(std::sync::atomic::Ordering::Relaxed))
        .map(|(i, _)| i)
        .collect();

    for &src in &dirty_source_indices {
        let total = domain.running_totals[src];
        if total > budget {
            let scale = budget / total;
            let mut new_total = 0.0;
            for &edge_idx in &domain.adjacency[src] {
                let edge = &mut domain.edges[edge_idx];
                if !edge.consolidated {
                    edge.conductance = (edge.conductance * scale).max(cond_min);
                }
                new_total += edge.conductance;
            }
            domain.running_totals[src] = new_total;
        }
    }
}
