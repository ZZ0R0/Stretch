use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::config::PropagationConfig;
use crate::domain::Domain;

/// Compute source contributions into a pre-allocated buffer.
/// Uses node_is_excitatory bitmap to avoid loading full Node struct for type check.
/// Returns the fired_list (indices of nodes with non-zero source_contrib).
pub fn compute_source_contribs(
    domain: &Domain,
    config: &PropagationConfig,
    gain_mods: &[f32],
    buf: &mut Vec<f32>,
    fired_list: &mut Vec<usize>,
) {
    let n = domain.nodes.len();
    buf.resize(n, 0.0);
    fired_list.clear();

    buf.par_iter_mut().enumerate().for_each(|(i, out)| {
        let node = &domain.nodes[i];
        if !node.is_active() {
            *out = 0.0;
            return;
        }
        let sign: f32 = if domain.node_is_excitatory[i] {
            1.0
        } else {
            -(config.gain_inhibitory as f32)
        };
        let gain_mod = 1.0 + gain_mods.get(i).copied().unwrap_or(0.0);
        *out = node.activation * sign * gain_mod * (config.gain as f32);
    });

    // Build fired_list from non-zero contribs
    for (i, &val) in buf.iter().enumerate() {
        if val != 0.0 {
            fired_list.push(i);
        }
    }
}

/// C1: Fired-only propagation using outgoing CSR.
/// Only iterates edges from nodes that actually fired (non-zero source_contrib).
/// Falls back to CSR incoming when fired_ratio > 30%.
pub fn compute_influences_fired_only(
    domain: &Domain,
    source_contribs: &[f32],
    fired_list: &[usize],
    influences_buf: &mut Vec<f32>,
) {
    let n = domain.nodes.len();
    influences_buf.resize(n, 0.0);

    let fired_ratio = fired_list.len() as f64 / n as f64;

    if fired_ratio > 0.30 || domain.outgoing.kernel_weights.is_empty() {
        // Fallback to incoming CSR (original method) — more efficient when many active
        compute_influences_csr(domain, source_contribs, influences_buf);
        return;
    }

    // Zero out influences
    for v in influences_buf.iter_mut() {
        *v = 0.0;
    }

    // Use atomic f32 accumulation via AtomicU32 bit-casting
    let atomic_buf: Vec<AtomicU32> = (0..n).map(|_| AtomicU32::new(0f32.to_bits())).collect();

    let outgoing = &domain.outgoing;
    let conductances = &domain.conductances;

    fired_list.par_iter().for_each(|&src_idx| {
        let contrib = source_contribs[src_idx];
        let start = outgoing.offsets[src_idx];
        let end = outgoing.offsets[src_idx + 1];
        for k in start..end {
            let target = outgoing.target_nodes[k];
            let val = contrib * conductances[outgoing.edge_indices[k]] * outgoing.kernel_weights[k];
            // Atomic f32 add via CAS loop
            atomic_f32_add(&atomic_buf[target], val);
        }
    });

    // Copy atomic results to output
    for (i, atomic) in atomic_buf.iter().enumerate() {
        influences_buf[i] = f32::from_bits(atomic.load(Ordering::Relaxed));
    }
}

/// Atomic f32 add using CAS loop (no std AtomicF32 in stable Rust).
#[inline]
fn atomic_f32_add(atomic: &AtomicU32, val: f32) {
    let mut current = atomic.load(Ordering::Relaxed);
    loop {
        let current_f32 = f32::from_bits(current);
        let new = (current_f32 + val).to_bits();
        match atomic.compare_exchange_weak(current, new, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(x) => current = x,
        }
    }
}

/// Compute influences using CSR incoming adjacency + conductance cache + kernel weights.
pub fn compute_influences_csr(
    domain: &Domain,
    source_contribs: &[f32],
    influences_buf: &mut Vec<f32>,
) {
    let n = domain.nodes.len();
    influences_buf.resize(n, 0.0);
    let csr = &domain.incoming;
    let conductances = &domain.conductances;

    influences_buf
        .par_iter_mut()
        .enumerate()
        .for_each(|(node_idx, out)| {
            let start = csr.offsets[node_idx];
            let end = csr.offsets[node_idx + 1];
            let mut total = 0.0_f32;
            for k in start..end {
                let src = source_contribs[csr.source_nodes[k]];
                if src == 0.0 { continue; }
                total += src * conductances[csr.edge_indices[k]] * csr.kernel_weights[k];
            }
            *out = total;
        });
}

/// Apply influences: update activation of target nodes.
/// Also updates node_needs_update bitmap (C5).
pub fn apply_influences(domain: &mut Domain, influences: &[f32]) -> usize {
    let count = std::sync::atomic::AtomicUsize::new(0);
    let threshold = 0.001_f32; // threshold for "meaningful influence"

    // Update node_needs_update bitmap based on influences
    domain.node_needs_update.par_iter_mut()
        .zip(influences.par_iter())
        .for_each(|(needs_update, &infl)| {
            if infl.abs() > threshold {
                *needs_update = true;
            }
        });

    domain
        .nodes
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, node)| {
            let was_active = node.is_active();
            node.activation = (node.activation + influences[i]).clamp(0.0, 10.0);
            if !was_active && node.is_active() {
                count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
    count.load(std::sync::atomic::Ordering::Relaxed)
}
