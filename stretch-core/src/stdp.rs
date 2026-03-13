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

/// Pre-compute spatial dopamine vec (only when reward_center changes).
pub fn precompute_spatial_dopamine(
    positions: &[[f64; 3]],
    center: [f64; 3],
    dopa_phasic: f64,
    spatial_lambda: f64,
    buf: &mut Vec<f64>,
) {
    let n = positions.len();
    buf.resize(n, 0.0);
    buf.par_iter_mut().enumerate().for_each(|(i, out)| {
        let pos = &positions[i];
        let dx = pos[0] - center[0];
        let dy = pos[1] - center[1];
        let dz = pos[2] - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        *out = dopa_phasic * (-spatial_lambda * dist).exp();
    });
}

/// Collect activation ticks snapshot into reusable buffer.
pub fn snapshot_activation_ticks(domain: &Domain, buf: &mut Vec<Option<usize>>) {
    let n = domain.nodes.len();
    buf.resize(n, None);
    // Sequential copy is faster than par_iter for 50k Option<usize>
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
    // Reusable buffers (managed by caller)
    activation_ticks_buf: &mut Vec<Option<usize>>,
    node_delta_dopa_buf: &mut Vec<f64>,
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

    let global_delta_dopa = dopa_phasic;

    let a_plus = stdp_config.a_plus;
    let a_minus = stdp_config.a_minus;
    let tau_plus = stdp_config.tau_plus;
    let tau_minus = stdp_config.tau_minus;

    let elig_decay = eligibility_config.decay;
    let elig_max = eligibility_config.max;

    let dopa_plasticity_gain = dopamine_config.plasticity_gain;
    let dopa_consolidation_threshold = dopamine_config.consolidation_threshold;

    let consolidation_threshold = consolidation.threshold;
    let consolidation_ticks = consolidation.ticks_required;

    let cond_min = edge_defaults.conductance_min;
    let cond_max = edge_defaults.conductance_max;
    let homeostatic_rate = edge_defaults.decay;
    let baseline_cond = edge_defaults.conductance;

    let activation_ticks = &*activation_ticks_buf;
    let node_delta_dopa = &*node_delta_dopa_buf;

    // Phase C : passage parallèle sur toutes les arêtes
    domain.edges.par_iter_mut().for_each(|edge| {
        // --- 1. STDP : calcul de la direction ψ ---
        let mut psi = 0.0;
        {
            let pre_tick = activation_ticks[edge.from];
            let post_tick = activation_ticks[edge.to];

            if let (Some(t_pre), Some(t_post)) = (pre_tick, post_tick) {
                if t_pre != t_post {
                    let dt = t_post as f64 - t_pre as f64;
                    psi = if dt > 0.0 {
                        a_plus * (-dt / tau_plus).exp()
                    } else {
                        -a_minus * (dt / tau_minus).exp()
                    };
                }
            }
        }

        // --- 2. Mise à jour de la trace d'éligibilité ---
        edge.eligibility = (elig_decay * edge.eligibility + psi).clamp(-elig_max, elig_max);

        // --- 3. Règle des trois facteurs avec dopamine locale ---
        {
            let delta_d = if use_spatial {
                // node_delta_dopa stores exp(-λd), multiply by actual phasic
                dopa_phasic * node_delta_dopa[edge.to]
            } else {
                global_delta_dopa
            };
            let dw = dopa_plasticity_gain * delta_d * edge.eligibility;
            edge.conductance = (edge.conductance + dw).clamp(cond_min, cond_max);
        }

        // --- 4. Décroissance homéostatique ---
        if !edge.consolidated {
            edge.conductance += homeostatic_rate * (baseline_cond - edge.conductance);
            edge.conductance = edge.conductance.clamp(cond_min, cond_max);
        }

        // --- 5. Consolidation ---
        if dopamine_level > dopa_consolidation_threshold && edge.eligibility > 0.0 {
            edge.update_consolidation(consolidation_threshold, consolidation_ticks);
        }
    });

    // Phase D : normalisation du budget synaptique par nœud source
    let budget = budget_config.budget;
    let totals: Vec<f64> = domain.adjacency.par_iter()
        .map(|adj| adj.iter().map(|&idx| domain.edges[idx].conductance).sum())
        .collect();
    domain.edges.par_iter_mut().for_each(|edge| {
        let total = totals[edge.from];
        if total > budget && !edge.consolidated {
            let scale = budget / total;
            edge.conductance = (edge.conductance * scale).max(cond_min);
        }
    });
}
