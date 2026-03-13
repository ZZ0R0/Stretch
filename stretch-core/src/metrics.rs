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
    pub global_energy: f64,
    pub max_activation: f64,
    pub mean_conductance: f64,
    pub max_conductance: f64,
    pub mean_memory_trace: f64,
    pub max_memory_trace: f64,
    pub mean_fatigue: f64,
    /// V2 : nombre d'arêtes consolidées
    pub consolidated_edges: usize,
    /// V2 : nombre de zones actives
    pub num_zones: usize,
    /// V2 : erreur PID moyenne (absolue)
    pub mean_pid_error: f64,
    /// V2 : sortie PID moyenne
    pub mean_pid_output: f64,
    /// V2 : activité moyenne des zones
    pub zone_activity_mean: f64,
    /// V3 : nombre de nœuds excitateurs actifs
    pub active_excitatory: usize,
    /// V3 : nombre de nœuds inhibiteurs actifs
    pub active_inhibitory: usize,
    /// V3 : énergie des nœuds excitateurs
    pub excitatory_energy: f64,
    /// V3 : énergie des nœuds inhibiteurs
    pub inhibitory_energy: f64,
    /// V4 : récompense courante
    pub current_reward: f64,
    /// V4 : récompense cumulée
    pub cumulative_reward: f64,
    /// V4 : niveau de dopamine
    pub dopamine_level: f64,
    /// V4 : trace d'éligibilité moyenne
    pub mean_eligibility: f64,
    /// V4 : dernière décision de sortie (None = pas de readout)
    pub output_decision: Option<usize>,
    /// V4 : accuracy courante
    pub accuracy: f64,
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
        cumulative_reward: f64,
        dopamine_level: f64,
        output_decision: Option<usize>,
        accuracy: f64,
    ) {
        let n_nodes = domain.nodes.len().max(1) as f64;
        let n_edges = domain.edges.len().max(1) as f64;

        // Parallel fold/reduce sur les nœuds
        let (active_nodes, global_energy, max_activation, sum_trace, max_trace, sum_fatigue,
             active_excitatory, active_inhibitory, excitatory_energy, inhibitory_energy) = domain
            .nodes
            .par_iter()
            .fold(
                || (0usize, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize, 0usize, 0.0f64, 0.0f64),
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
                || (0.0f64, 0.0f64, 0usize, 0.0f64),
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
            mean_pid_error: zone_mgr.mean_pid_error(),
            mean_pid_output: zone_mgr.mean_pid_output(),
            zone_activity_mean: zone_mgr.global_activity_mean(),
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
            accuracy,
        });
    }

    /// Top-N des liaisons les plus utilisées.
    pub fn top_edges(domain: &Domain, n: usize) -> Vec<(usize, usize, u64, f64)> {
        let mut edge_stats: Vec<(usize, usize, u64, f64)> = domain
            .edges
            .iter()
            .map(|e| (e.from, e.to, e.usage_count, e.conductance))
            .collect();
        edge_stats.sort_by(|a, b| b.2.cmp(&a.2));
        edge_stats.truncate(n);
        edge_stats
    }

    /// Distribution des traces mémoire (histogramme).
    pub fn trace_histogram(domain: &Domain, bins: usize) -> Vec<(f64, usize)> {
        let max_trace = domain
            .nodes
            .iter()
            .map(|n| n.memory_trace)
            .fold(0.0_f64, f64::max)
            .max(0.001);
        let bin_width = max_trace / bins as f64;

        let mut histogram: HashMap<usize, usize> = HashMap::new();
        for node in &domain.nodes {
            let bin = ((node.memory_trace / bin_width) as usize).min(bins - 1);
            *histogram.entry(bin).or_insert(0) += 1;
        }

        (0..bins)
            .map(|b| (b as f64 * bin_width, *histogram.get(&b).unwrap_or(&0)))
            .collect()
    }
}
