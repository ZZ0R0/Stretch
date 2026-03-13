use rayon::prelude::*;

use crate::config::{ConsolidationConfig, EdgeDefaults, PlasticityConfig, StdpConfig, SynapticBudgetConfig};
use crate::domain::Domain;

/// V3 optimisé : Plasticité hebbienne + STDP + budget synaptique en un seul passage parallèle.
///
/// Fusionne les 3 opérations sur les arêtes pour réduire le nombre de passes
/// sur le tableau d'arêtes (~37 MB), dont l'itération est memory-bandwidth-bound.
///
/// Ordre : totaux budget (lecture seule) → puis PLAST + STDP + budget apply (lecture+écriture).
pub fn update_plasticity_stdp_budget(
    domain: &mut Domain,
    plasticity: &PlasticityConfig,
    consolidation: &ConsolidationConfig,
    stdp_config: &StdpConfig,
    budget_config: &SynapticBudgetConfig,
    edge_defaults: &EdgeDefaults,
    tick: usize,
) {
    // Phase A : mise à jour des last_activation_tick (STDP — parallèle sur les nœuds)
    if stdp_config.enabled {
        domain.nodes.par_iter_mut().for_each(|node| {
            if node.is_active() {
                node.last_activation_tick = Some(tick);
            }
        });
    }

    // Phase B : collecter les données des nœuds (évite les problèmes d'emprunt)
    let activations: Vec<f64> = domain.nodes.iter().map(|n| n.activation).collect();
    let activation_ticks: Vec<Option<usize>> = if stdp_config.enabled {
        domain.nodes.iter().map(|n| n.last_activation_tick).collect()
    } else {
        Vec::new()
    };

    // Phase B2 : calculer les totaux de conductance par nœud source (pour le budget)
    // Utilise les conductances du tick précédent — fonctionnellement équivalent
    let budget_enabled = budget_config.enabled;
    let budget = budget_config.budget;
    let budget_totals: Vec<f64> = if budget_enabled {
        domain
            .adjacency
            .par_iter()
            .map(|adj_list| {
                adj_list
                    .iter()
                    .map(|&edge_idx| domain.edges[edge_idx].conductance)
                    .sum()
            })
            .collect()
    } else {
        Vec::new()
    };

    let coactivity_decay = plasticity.coactivity_decay;
    let reinforcement_rate = plasticity.reinforcement_rate;
    let weakening_rate = plasticity.weakening_rate;
    let coactivation_threshold = plasticity.coactivation_threshold;
    let consolidation_enabled = consolidation.enabled;
    let consolidation_threshold = consolidation.threshold;
    let consolidation_ticks = consolidation.ticks_required;

    let stdp_enabled = stdp_config.enabled;
    let a_plus = stdp_config.a_plus;
    let a_minus = stdp_config.a_minus;
    let tau_plus = stdp_config.tau_plus;
    let tau_minus = stdp_config.tau_minus;

    let edge_plasticity = edge_defaults.plasticity;
    let cond_min = edge_defaults.conductance_min;
    let cond_max = edge_defaults.conductance_max;

    // Phase C : passage UNIQUE parallèle sur toutes les arêtes
    // Fusionne plasticité + STDP + normalisation budget
    domain.edges.par_iter_mut().for_each(|edge| {
        // --- Plasticité hebbienne ---
        let src_act = activations[edge.from];
        let tgt_act = activations[edge.to];

        if src_act > 0.1 && tgt_act > 0.1 {
            edge.record_coactivation(src_act, tgt_act);
        }
        edge.decay_coactivity(coactivity_decay);
        edge.update_conductance(reinforcement_rate, weakening_rate, coactivation_threshold, edge_plasticity, cond_min, cond_max);

        if consolidation_enabled {
            edge.update_consolidation(consolidation_threshold, consolidation_ticks);
        }

        // --- STDP ---
        if stdp_enabled {
            let pre_tick = activation_ticks[edge.from];
            let post_tick = activation_ticks[edge.to];

            if let (Some(t_pre), Some(t_post)) = (pre_tick, post_tick) {
                if t_pre != t_post {
                    let dt = t_post as f64 - t_pre as f64;
                    let dw = if dt > 0.0 {
                        a_plus * (-dt / tau_plus).exp()
                    } else {
                        -a_minus * (dt / tau_minus).exp()
                    };
                    edge.conductance =
                        (edge.conductance + dw).clamp(cond_min, cond_max);
                }
            }
        }

        // --- Budget synaptique ---
        if budget_enabled {
            let total = budget_totals[edge.from];
            if total > budget && !edge.consolidated {
                let scale = budget / total;
                edge.conductance = (edge.conductance * scale).max(cond_min);
            }
        }
    });
}
