use crate::config::PropagationConfig;
use crate::domain::Domain;

/// Calculer l'influence reçue par chaque nœud à ce tick.
/// Seuls les nœuds actifs (activation > seuil effectif) propagent.
pub fn compute_influences(domain: &Domain, config: &PropagationConfig) -> Vec<f64> {
    let n = domain.num_nodes();
    let mut influences = vec![0.0_f64; n];

    for (i, adj_list) in domain.adjacency.iter().enumerate() {
        let source = &domain.nodes[i];
        if !source.is_active() {
            continue;
        }

        for &edge_idx in adj_list {
            let edge = &domain.edges[edge_idx];
            let target_id = edge.to;

            // Noyau de propagation : décroissance spatiale
            let kernel_value = match config.kernel.as_str() {
                "exponential" => (-config.spatial_decay * edge.distance).exp(),
                "gaussian" => (-0.5 * (edge.distance * config.spatial_decay).powi(2)).exp(),
                _ => (-config.spatial_decay * edge.distance).exp(),
            };

            // Influence = activation_source * conductance * noyau * gain
            let influence = source.activation * edge.conductance * kernel_value * config.gain;

            influences[target_id] += influence;
        }
    }

    influences
}

/// Appliquer les influences : mettre à jour l'activation des nœuds cibles.
/// Retourne les indices des nœuds nouvellement activés.
pub fn apply_influences(domain: &mut Domain, influences: &[f64]) -> Vec<usize> {
    let mut newly_activated = Vec::new();

    for i in 0..domain.nodes.len() {
        let was_active = domain.nodes[i].is_active();
        let influence = influences[i];

        if influence > 0.0 {
            domain.nodes[i].activation += influence;
            // Bornage
            domain.nodes[i].activation = domain.nodes[i].activation.min(10.0);
        }

        if !was_active && domain.nodes[i].is_active() {
            newly_activated.push(i);
        }
    }

    newly_activated
}
