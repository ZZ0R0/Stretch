/// V6 : Sparsité à front d'onde (wavefront-aware sparsity).
///
/// Sélectionne les top-K neurones par score (activation × bonus_nouveauté)
/// et supprime les autres. Le seuil est calculé sur CPU à chaque tick
/// et passé au GPU via GpuParams.

use crate::config::V6SparsityConfig;

/// Compute the activation threshold for sparsity enforcement.
///
/// Returns the threshold value such that at most `max_active_count` neurons
/// exceed it (based on novelty-weighted scores from the previous tick).
///
/// `activations`: current per-node activations
/// `first_activation_tick`: per-node tick of first activation (u32::MAX = never)
/// `current_tick`: current simulation tick
/// `config`: sparsity configuration
pub fn compute_sparsity_threshold(
    activations: &[f32],
    first_activation_tick: &[u32],
    current_tick: u32,
    config: &V6SparsityConfig,
) -> f32 {
    let n = activations.len();
    let max_active = ((n as f64) * config.max_active_fraction) as usize;
    if max_active >= n {
        return 0.0;
    }

    // Compute novelty-weighted scores
    let novelty_gain = config.novelty_gain as f32;
    let novelty_window = config.novelty_window as f32;

    let mut scores: Vec<f32> = activations
        .iter()
        .zip(first_activation_tick.iter())
        .map(|(&act, &first_tick)| {
            if act < 0.01 {
                return 0.0;
            }
            let bonus = if first_tick < u32::MAX {
                let age = current_tick.saturating_sub(first_tick) as f32;
                let raw = (novelty_window - age).max(0.0) / novelty_window;
                1.0 + novelty_gain * raw
            } else {
                1.0 + novelty_gain // never activated → full bonus
            };
            act * bonus
        })
        .collect();

    // Partial sort to find the threshold (K-th largest element)
    let k = n - max_active;
    scores.select_nth_unstable_by(k, |a, b| a.partial_cmp(b).unwrap());
    scores[k]
}

/// Apply sparsity enforcement on CPU: suppress neurons below threshold.
///
/// Also updates `first_activation_tick` for newly activated neurons.
pub fn apply_sparsity_cpu(
    activations: &mut [f32],
    first_activation_tick: &mut [u32],
    thresholds: &[f32],
    fatigues: &[f32],
    inhibitions: &[f32],
    excitabilities: &[f32],
    threshold_mods: &[f32],
    current_tick: u32,
    config: &V6SparsityConfig,
    sparsity_threshold: f32,
) {
    let novelty_gain = config.novelty_gain as f32;
    let novelty_window = config.novelty_window as f32;
    let suppress = config.suppress_factor as f32;

    for i in 0..activations.len() {
        let act = activations[i];
        if act < 0.01 {
            continue;
        }

        // Check if node is "active" (above its effective threshold)
        let eff_threshold = ((thresholds[i] + fatigues[i] + inhibitions[i] + threshold_mods[i])
            / excitabilities[i].max(0.01))
            .max(0.05);

        if act > eff_threshold && first_activation_tick[i] == u32::MAX {
            first_activation_tick[i] = current_tick;
        }

        // Compute novelty score
        let bonus = if first_activation_tick[i] < u32::MAX {
            let age = current_tick.saturating_sub(first_activation_tick[i]) as f32;
            let raw = (novelty_window - age).max(0.0) / novelty_window;
            1.0 + novelty_gain * raw
        } else {
            1.0
        };
        let score = act * bonus;

        // Suppress if below threshold
        if score < sparsity_threshold {
            activations[i] *= suppress;
        }
    }
}

/// Reset first_activation_tick buffer (at trial start).
pub fn reset_first_activation_tick(buf: &mut [u32]) {
    buf.fill(u32::MAX);
}
