//! V5 : Calibration multi-échelle.
//!
//! Ajuste les hyperparamètres en fonction de la taille du réseau `n`
//! selon des lois adaptatives simples.

use crate::config::SimConfig;

/// Applique la calibration multi-échelle sur la config.
/// Modifie en place les paramètres qui dépendent de la taille du réseau.
pub fn apply_calibration(config: &mut SimConfig) {
    let cal = &config.v5_calibration;
    if !cal.enabled {
        return;
    }

    let n = config.domain.size as f64;
    let n_ref = cal.ref_n as f64;
    let g_ref = cal.ref_gain;

    // group_size(n) = max(group_size_min, sqrt(n))
    let adapted_group_size = (n.sqrt() as usize).max(cal.group_size_min);
    config.input.group_size = adapted_group_size;
    config.output.group_size = adapted_group_size;

    // gain(n) = g_ref * log(n) / log(n_ref)
    let adapted_gain = g_ref * n.ln() / n_ref.ln();
    config.propagation.gain = adapted_gain;
    config.propagation.gain_inhibitory = adapted_gain;

    // eligibility_decay(n) ∈ [0.85, 0.98] — plus lent pour réseaux plus grands
    let elig_decay = (0.85 + 0.13 * (n.ln() / n_ref.ln() - 1.0).max(0.0).min(1.0)).min(0.98);
    config.eligibility.decay = elig_decay;

    // edge_decay(n) = edge_decay_ref / (n / n_ref) — plus lent quand n augmente
    let edge_decay_ref = config.edge_defaults.decay;
    let adapted_edge_decay = edge_decay_ref / (n / n_ref);
    config.edge_defaults.decay = adapted_edge_decay;

    // target_activity(n) = min(target_ref, K / n) — activité plus sparse quand n grand
    let k_activity = config.zones.target_activity * n_ref;
    let adapted_target = (config.zones.target_activity).min(k_activity / n);
    config.zones.target_activity = adapted_target;

    eprintln!("[V5 Calibration] n={}, group_size={}, gain={:.4}, elig_decay={:.3}, edge_decay={:.6}, target_act={:.4}",
        config.domain.size, adapted_group_size, adapted_gain, elig_decay, adapted_edge_decay, adapted_target);
}
