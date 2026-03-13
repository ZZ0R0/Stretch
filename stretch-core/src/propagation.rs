use rayon::prelude::*;

use crate::config::PropagationConfig;
use crate::domain::Domain;
use crate::node::NeuronType;

/// Calculer l'influence reçue par chaque nœud à ce tick (parallèle, target-centric).
/// V3 : propagation signée — les neurones excitateurs propagent positivement,
/// les neurones inhibiteurs propagent négativement (× gain_inhibitory).
/// Utilise incoming_adjacency pour paralléliser par nœud cible sans contention.
pub fn compute_influences(
    domain: &Domain,
    config: &PropagationConfig,
    gain_mods: &[f64],
) -> Vec<f64> {
    let is_exponential = config.kernel == "exponential";
    let spatial_decay = config.spatial_decay;

    // Pré-calculer la contribution de chaque source (activation × sign × gain_mod × gain)
    // Cela évite de recalculer ces valeurs pour chaque arête sortante.
    let source_contribs: Vec<f64> = domain
        .nodes
        .par_iter()
        .enumerate()
        .map(|(i, node)| {
            if !node.is_active() {
                return 0.0;
            }
            let sign = match node.node_type {
                NeuronType::Excitatory => 1.0,
                NeuronType::Inhibitory => -config.gain_inhibitory,
            };
            let gain_mod = 1.0 + gain_mods.get(i).copied().unwrap_or(0.0);
            node.activation * sign * gain_mod * config.gain
        })
        .collect();

    domain
        .incoming_adjacency
        .par_iter()
        .map(|in_edges| {
            let mut total = 0.0_f64;
            for &edge_idx in in_edges {
                let edge = &domain.edges[edge_idx];
                let src = source_contribs[edge.from];
                if src == 0.0 {
                    continue;
                }

                let kernel_value = if is_exponential {
                    (-spatial_decay * edge.distance).exp()
                } else {
                    (-0.5 * (edge.distance * spatial_decay).powi(2)).exp()
                };

                total += src * edge.conductance * kernel_value;
            }
            total
        })
        .collect()
}

/// Appliquer les influences (parallèle) : mettre à jour l'activation des nœuds cibles.
/// V3 : les influences négatives (inhibition) sont appliquées aussi.
/// Retourne les indices des nœuds nouvellement activés.
pub fn apply_influences(domain: &mut Domain, influences: &[f64]) -> Vec<usize> {
    let newly_activated: Vec<usize> = domain
        .nodes
        .par_iter_mut()
        .enumerate()
        .filter_map(|(i, node)| {
            let was_active = node.is_active();
            node.activation = (node.activation + influences[i]).clamp(0.0, 10.0);
            if !was_active && node.is_active() {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    newly_activated
}
