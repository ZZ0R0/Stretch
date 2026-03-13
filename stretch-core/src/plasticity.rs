use crate::config::{ConsolidationConfig, PlasticityConfig};
use crate::domain::Domain;

/// Mettre à jour les traces de co-activation et les conductances.
pub fn update_plasticity(domain: &mut Domain, config: &PlasticityConfig, consolidation: &ConsolidationConfig) {
    // Phase 1 : collecter les activations pour éviter les problèmes de borrow
    let activations: Vec<f64> = domain.nodes.iter().map(|n| n.activation).collect();

    // Phase 2 : mettre à jour les liaisons
    for edge in domain.edges.iter_mut() {
        let src_act = activations[edge.from];
        let tgt_act = activations[edge.to];

        // Enregistrer co-activation si les deux nœuds sont suffisamment actifs
        if src_act > 0.1 && tgt_act > 0.1 {
            edge.record_coactivation(src_act, tgt_act);
        }

        // Décroissance de la trace de co-activation
        edge.decay_coactivity(config.coactivity_decay);

        // Mise à jour de la conductance (renforcement / affaiblissement)
        edge.update_conductance(
            config.reinforcement_rate,
            config.weakening_rate,
            config.coactivation_threshold,
        );

        // V2 : consolidation mémoire
        if consolidation.enabled {
            edge.update_consolidation(consolidation.threshold, consolidation.ticks_required);
        }
    }
}
