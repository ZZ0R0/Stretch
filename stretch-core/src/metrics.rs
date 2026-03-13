use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::Domain;
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

    pub fn record(&mut self, tick: usize, domain: &Domain, zone_mgr: &ZoneManager) {
        let active_nodes = domain.nodes.iter().filter(|n| n.is_active()).count();
        let global_energy: f64 = domain.nodes.iter().map(|n| n.activation).sum();
        let max_activation = domain
            .nodes
            .iter()
            .map(|n| n.activation)
            .fold(0.0_f64, f64::max);

        let n_edges = domain.edges.len().max(1) as f64;
        let mean_conductance: f64 = domain.edges.iter().map(|e| e.conductance).sum::<f64>() / n_edges;
        let max_conductance = domain
            .edges
            .iter()
            .map(|e| e.conductance)
            .fold(0.0_f64, f64::max);

        let n_nodes = domain.nodes.len().max(1) as f64;
        let mean_memory_trace: f64 =
            domain.nodes.iter().map(|n| n.memory_trace).sum::<f64>() / n_nodes;
        let max_memory_trace = domain
            .nodes
            .iter()
            .map(|n| n.memory_trace)
            .fold(0.0_f64, f64::max);
        let mean_fatigue: f64 = domain.nodes.iter().map(|n| n.fatigue).sum::<f64>() / n_nodes;

        let consolidated_edges = domain.edges.iter().filter(|e| e.consolidated).count();

        self.snapshots.push(TickMetrics {
            tick,
            active_nodes,
            global_energy,
            max_activation,
            mean_conductance,
            max_conductance,
            mean_memory_trace,
            max_memory_trace,
            mean_fatigue,
            consolidated_edges,
            num_zones: zone_mgr.num_zones(),
            mean_pid_error: zone_mgr.mean_pid_error(),
            mean_pid_output: zone_mgr.mean_pid_output(),
            zone_activity_mean: zone_mgr.global_activity_mean(),
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
