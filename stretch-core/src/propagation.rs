use rayon::prelude::*;

use crate::config::PropagationConfig;
use crate::domain::Domain;

/// Compute source contributions into a pre-allocated buffer.
/// Uses node_is_excitatory bitmap to avoid loading full Node struct for type check.
pub fn compute_source_contribs(
    domain: &Domain,
    config: &PropagationConfig,
    gain_mods: &[f64],
    buf: &mut Vec<f64>,
) {
    let n = domain.nodes.len();
    buf.resize(n, 0.0);

    buf.par_iter_mut().enumerate().for_each(|(i, out)| {
        let node = &domain.nodes[i];
        if !node.is_active() {
            *out = 0.0;
            return;
        }
        let sign = if domain.node_is_excitatory[i] {
            1.0
        } else {
            -config.gain_inhibitory
        };
        let gain_mod = 1.0 + gain_mods.get(i).copied().unwrap_or(0.0);
        *out = node.activation * sign * gain_mod * config.gain;
    });
}

/// Compute influences using CSR incoming adjacency + conductance cache + kernel weights.
/// All hot data is contiguous or cache-friendly:
///   - kernel_weights: sequential in CSR (co-located)
///   - source_nodes: sequential in CSR (co-located)
///   - conductances: 4.6MB separate cache (vs 55MB edge array)
pub fn compute_influences_csr(
    domain: &Domain,
    source_contribs: &[f64],
    influences_buf: &mut Vec<f64>,
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
            let mut total = 0.0_f64;
            for k in start..end {
                let src = source_contribs[csr.source_nodes[k]];
                if src == 0.0 { continue; }
                total += src * conductances[csr.edge_indices[k]] * csr.kernel_weights[k];
            }
            *out = total;
        });
}

/// Apply influences: update activation of target nodes.
/// Returns count of newly activated nodes.
pub fn apply_influences(domain: &mut Domain, influences: &[f64]) -> usize {
    let count = std::sync::atomic::AtomicUsize::new(0);
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
