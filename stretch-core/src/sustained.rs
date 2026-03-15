//! V5 : Dynamique d'activité soutenue.
//!
//! Mécanismes pour casser le régime impulsionnel flash & die de V4 :
//! - Decay adaptatif : le decay diminue quand l'activité locale est élevée
//! - Réverbération locale : feedback court-terme de l'activation précédente
//! - Reset partiel : ne pas écraser complètement l'activité entre trials

use rayon::prelude::*;

use crate::domain::Domain;

/// Applique le decay adaptatif V5 au lieu du decay fixe V4.
///
/// α_eff_i(t) = α_base × (1 − k_local × mean_neighbor_activity_i)
/// Borné dans [α_base × 0.2, α_base] pour éviter l'arrêt total du decay.
pub fn apply_adaptive_decay(
    domain: &mut Domain,
    base_decay: f32,
    k_local: f32,
    activation_min: f32,
) {
    // Calcul de l'activité locale moyenne par nœud (via les voisins sortants CSR)
    let local_activities: Vec<f32> = (0..domain.num_nodes())
        .into_par_iter()
        .map(|i| {
            let out_start = domain.outgoing.offsets[i];
            let out_end = domain.outgoing.offsets[i + 1];
            if out_start == out_end {
                return 0.0;
            }
            let mut sum = 0.0;
            let mut count = 0;
            for k in out_start..out_end {
                let target = domain.outgoing.target_nodes[k];
                sum += domain.nodes[target].activation;
                count += 1;
            }
            if count > 0 { sum / count as f32 } else { 0.0 }
        })
        .collect();

    // Application du decay adaptatif
    domain.nodes.par_iter_mut()
        .zip(local_activities.par_iter())
        .for_each(|(node, &local_act)| {
            if node.activation <= activation_min {
                return;
            }
            let factor = (1.0 - k_local * local_act).clamp(0.2, 1.0);
            let effective_decay = base_decay * factor;
            let new_act = node.activation * (1.0 - effective_decay);
            node.activation = new_act.max(activation_min);
        });
}

/// Applique la réverbération locale : ajoute une fraction de l'activation précédente.
///
/// Utilise un buffer externe pour stocker les activations du tick précédent.
/// phi_eff = phi + r_local * phi_prev (borné à [0, activation_max])
pub fn apply_reverberation(
    domain: &mut Domain,
    prev_activations: &[f32],
    reverb_gain: f32,
) {
    let activation_max: f32 = 5.0; // sécurité
    domain.nodes.par_iter_mut()
        .zip(prev_activations.par_iter())
        .for_each(|(node, &prev)| {
            let reverb = reverb_gain * prev;
            if reverb > 0.001 {
                node.activation = (node.activation + reverb).min(activation_max);
            }
        });
}

/// Snapshot des activations courantes (pour la réverbération du tick suivant).
pub fn snapshot_activations(domain: &Domain, buf: &mut Vec<f32>) {
    buf.resize(domain.num_nodes(), 0.0);
    for (i, node) in domain.nodes.iter().enumerate() {
        buf[i] = node.activation;
    }
}

/// Applique la politique de reset V5 au début d'un trial.
pub fn apply_reset_policy(domain: &mut Domain, policy: &str) {
    match policy {
        "full" => {
            // V4 classique : tout à zéro
            for node in domain.nodes.iter_mut() {
                node.activation = 0.0;
            }
        }
        "partial" => {
            // V5 : garde 10% de l'activation résiduelle
            for node in domain.nodes.iter_mut() {
                node.activation *= 0.1;
            }
        }
        "none" => {
            // Pas de reset — l'activité continue librement
        }
        _ => {
            // Fallback : full reset
            for node in domain.nodes.iter_mut() {
                node.activation = 0.0;
            }
        }
    }
}
