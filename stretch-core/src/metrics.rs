use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::Domain;
use crate::node::NeuronType;
use crate::zone::ZoneManager;

/// Snapshot des métriques à un instant donné.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickMetrics {
    pub tick: usize,
    pub active_nodes: usize,
    pub global_energy: f32,
    pub max_activation: f32,
    pub mean_conductance: f32,
    pub max_conductance: f32,
    pub mean_memory_trace: f32,
    pub max_memory_trace: f32,
    pub mean_fatigue: f32,
    /// V2 : nombre d'arêtes consolidées
    pub consolidated_edges: usize,
    /// V2 : nombre de zones actives
    pub num_zones: usize,
    /// V2 : erreur PID moyenne (absolue)
    pub mean_pid_error: f32,
    /// V2 : sortie PID moyenne
    pub mean_pid_output: f32,
    /// V2 : activité moyenne des zones
    pub zone_activity_mean: f32,
    /// V3 : nombre de nœuds excitateurs actifs
    pub active_excitatory: usize,
    /// V3 : nombre de nœuds inhibiteurs actifs
    pub active_inhibitory: usize,
    /// V3 : énergie des nœuds excitateurs
    pub excitatory_energy: f32,
    /// V3 : énergie des nœuds inhibiteurs
    pub inhibitory_energy: f32,
    /// V4 : récompense courante
    pub current_reward: f32,
    /// V4 : récompense cumulée
    pub cumulative_reward: f32,
    /// V4 : niveau de dopamine
    pub dopamine_level: f32,
    /// V4 : trace d'éligibilité moyenne
    pub mean_eligibility: f32,
    /// V4 : dernière décision de sortie (None = pas de readout)
    pub output_decision: Option<usize>,
    /// V4 : accuracy courante
    pub accuracy: f32,
    /// V5.2 : baseline RPE (moyenne glissante du reward)
    #[serde(default)]
    pub rpe_baseline: f32,
    /// V5.2 : RPE delta (δ = r_eff - baseline)
    #[serde(default)]
    pub rpe_delta: f32,
}

/// Collecte complète des métriques sur la simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsLog {
    pub snapshots: Vec<TickMetrics>,
}

impl MetricsLog {
    pub fn new() -> Self {
        MetricsLog {
            snapshots: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        tick: usize,
        domain: &Domain,
        zone_mgr: &ZoneManager,
        cumulative_reward: f32,
        dopamine_level: f32,
        output_decision: Option<usize>,
        accuracy: f64,
        rpe_baseline: f32,
        rpe_delta: f32,
    ) {
        let n_nodes = domain.nodes.len().max(1) as f32;
        let n_edges = domain.edges.len().max(1) as f32;

        // Parallel fold/reduce sur les nœuds
        let (active_nodes, global_energy, max_activation, sum_trace, max_trace, sum_fatigue,
             active_excitatory, active_inhibitory, excitatory_energy, inhibitory_energy) = domain
            .nodes
            .par_iter()
            .fold(
                || (0usize, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0usize, 0usize, 0.0f32, 0.0f32),
                |mut acc, node| {
                    let active = node.is_active();
                    if active { acc.0 += 1; }
                    acc.1 += node.activation;
                    if node.activation > acc.2 { acc.2 = node.activation; }
                    acc.3 += node.memory_trace;
                    if node.memory_trace > acc.4 { acc.4 = node.memory_trace; }
                    acc.5 += node.fatigue;
                    match node.node_type {
                        NeuronType::Excitatory => {
                            acc.8 += node.activation;
                            if active { acc.6 += 1; }
                        }
                        NeuronType::Inhibitory => {
                            acc.9 += node.activation;
                            if active { acc.7 += 1; }
                        }
                    }
                    acc
                },
            )
            .reduce(
                || (0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, 0.0, 0.0),
                |a, b| (
                    a.0 + b.0, a.1 + b.1, a.2.max(b.2), a.3 + b.3, a.4.max(b.4),
                    a.5 + b.5, a.6 + b.6, a.7 + b.7, a.8 + b.8, a.9 + b.9,
                ),
            );

        // Parallel fold/reduce sur les arêtes (conductance + consolidation + éligibilité)
        let (sum_cond, max_conductance, consolidated_edges, sum_eligibility) = domain
            .edges
            .par_iter()
            .fold(
                || (0.0f32, 0.0f32, 0usize, 0.0f32),
                |mut acc, e| {
                    acc.0 += e.conductance;
                    if e.conductance > acc.1 { acc.1 = e.conductance; }
                    if e.consolidated { acc.2 += 1; }
                    acc.3 += e.eligibility.abs();
                    acc
                },
            )
            .reduce(
                || (0.0, 0.0, 0, 0.0),
                |a, b| (a.0 + b.0, a.1.max(b.1), a.2 + b.2, a.3 + b.3),
            );

        self.snapshots.push(TickMetrics {
            tick,
            active_nodes,
            global_energy,
            max_activation,
            mean_conductance: sum_cond / n_edges,
            max_conductance,
            mean_memory_trace: sum_trace / n_nodes,
            max_memory_trace: max_trace,
            mean_fatigue: sum_fatigue / n_nodes,
            consolidated_edges,
            num_zones: zone_mgr.num_zones(),
            mean_pid_error: zone_mgr.mean_pid_error() as f32,
            mean_pid_output: zone_mgr.mean_pid_output() as f32,
            zone_activity_mean: zone_mgr.global_activity_mean() as f32,
            active_excitatory,
            active_inhibitory,
            excitatory_energy,
            inhibitory_energy,
            // V4
            current_reward: 0.0,
            cumulative_reward,
            dopamine_level,
            mean_eligibility: sum_eligibility / n_edges,
            output_decision,
            accuracy: accuracy as f32,
            // V5.2
            rpe_baseline,
            rpe_delta,
        });
    }

    /// Record metrics from GPU-side reduction (avoids full state download).
    pub fn record_from_gpu(
        &mut self,
        tick: usize,
        gpu_metrics: &crate::gpu::GpuMetrics,
        zone_mgr: &ZoneManager,
        cumulative_reward: f32,
        dopamine_level: f32,
        output_decision: Option<usize>,
        accuracy: f64,
        rpe_baseline: f32,
        rpe_delta: f32,
    ) {
        self.snapshots.push(TickMetrics {
            tick,
            active_nodes: gpu_metrics.active_count as usize,
            global_energy: gpu_metrics.global_energy,
            max_activation: gpu_metrics.max_activation,
            mean_conductance: gpu_metrics.mean_conductance,
            max_conductance: gpu_metrics.max_conductance,
            mean_memory_trace: gpu_metrics.mean_memory_trace,
            max_memory_trace: gpu_metrics.max_memory_trace,
            mean_fatigue: gpu_metrics.mean_fatigue,
            consolidated_edges: gpu_metrics.consolidated_count as usize,
            num_zones: zone_mgr.num_zones(),
            mean_pid_error: zone_mgr.mean_pid_error() as f32,
            mean_pid_output: zone_mgr.mean_pid_output() as f32,
            zone_activity_mean: zone_mgr.global_activity_mean() as f32,
            active_excitatory: gpu_metrics.active_excitatory as usize,
            active_inhibitory: gpu_metrics.active_inhibitory as usize,
            excitatory_energy: gpu_metrics.excitatory_energy,
            inhibitory_energy: gpu_metrics.inhibitory_energy,
            current_reward: 0.0,
            cumulative_reward,
            dopamine_level,
            mean_eligibility: gpu_metrics.mean_eligibility,
            output_decision,
            accuracy: accuracy as f32,
            // V5.2
            rpe_baseline,
            rpe_delta,
        });
    }

    /// Top-N des liaisons les plus utilisées.
    pub fn top_edges(domain: &Domain, n: usize) -> Vec<(usize, usize, u64, f32)> {
        let mut edge_stats: Vec<(usize, usize, u64, f32)> = domain
            .edges
            .iter()
            .map(|e| (e.from, e.to, e.usage_count, e.conductance))
            .collect();
        edge_stats.sort_by(|a, b| b.2.cmp(&a.2));
        edge_stats.truncate(n);
        edge_stats
    }

    /// Distribution des traces mémoire (histogramme).
    pub fn trace_histogram(domain: &Domain, bins: usize) -> Vec<(f32, usize)> {
        let max_trace = domain
            .nodes
            .iter()
            .map(|n| n.memory_trace)
            .fold(0.0_f32, f32::max)
            .max(0.001);
        let bin_width = max_trace / bins as f32;

        let mut histogram: HashMap<usize, usize> = HashMap::new();
        for node in &domain.nodes {
            let bin = ((node.memory_trace / bin_width) as usize).min(bins - 1);
            *histogram.entry(bin).or_insert(0) += 1;
        }

        (0..bins)
            .map(|b| (b as f32 * bin_width, *histogram.get(&b).unwrap_or(&0)))
            .collect()
    }
}
