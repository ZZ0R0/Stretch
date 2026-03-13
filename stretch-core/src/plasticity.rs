use rayon::prelude::*;

use crate::config::{ConsolidationConfig, EdgeDefaults, PlasticityConfig};
use crate::domain::Domain;

/// Mettre à jour les traces de co-activation et les conductances (parallèle).
/// Note: en V3, utiliser `stdp::update_plasticity_and_stdp` pour fusionner avec STDP.
pub fn update_plasticity(domain: &mut Domain, config: &PlasticityConfig, consolidation: &ConsolidationConfig, edge_defaults: &EdgeDefaults) {
    let activations: Vec<f64> = domain.nodes.iter().map(|n| n.activation).collect();

    let coactivity_decay = config.coactivity_decay;
    let reinforcement_rate = config.reinforcement_rate;
    let weakening_rate = config.weakening_rate;
    let coactivation_threshold = config.coactivation_threshold;
    let consolidation_enabled = consolidation.enabled;
    let consolidation_threshold = consolidation.threshold;
    let consolidation_ticks = consolidation.ticks_required;
    let edge_plasticity = edge_defaults.plasticity;
    let cond_min = edge_defaults.conductance_min;
    let cond_max = edge_defaults.conductance_max;

    domain.edges.par_iter_mut().for_each(|edge| {
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
    });
}
