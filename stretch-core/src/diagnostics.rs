//! V5 : Système de diagnostics pour la preuve d'apprentissage.
//!
//! Fournit :
//! - Path tracer : chemins de conductance maximale entre groupes I/O
//! - RouteScore : score de route entre un input et un output
//! - Indice directionnel D_k : différence de route entre cible et compétiteur
//! - Cohérence topologique CT : corrélation entre ΔC et appartenance aux routes utiles
//! - Sustain ratio : rapport énergie inter-trial / énergie pic

use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;

use crate::domain::Domain;

// =========================================================================
// Path Tracer — Dijkstra modifié sur conductance (chemin de C maximale)
// =========================================================================

#[derive(Clone, PartialEq)]
struct PathState {
    node: usize,
    cost: f64, // -log(conductance) pour transformation en chemin le plus court
}

impl Eq for PathState {}

impl Ord for PathState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Résultat d'un path trace.
#[derive(Debug, Clone)]
pub struct TracedPath {
    /// Séquence d'indices de nœuds du chemin
    pub nodes: Vec<usize>,
    /// Séquence d'indices d'arêtes du chemin
    pub edges: Vec<usize>,
    /// Score de route = somme des conductances sur le chemin
    pub route_score: f64,
    /// Conductance minimale le long du chemin (bottleneck)
    pub min_conductance: f64,
}

/// Trouve le chemin de conductance maximale entre un ensemble source et un ensemble cible.
///
/// Utilise Dijkstra sur -log(conductance) pour trouver le chemin le plus "conducteur".
/// Les arêtes avec conductance < min_cond sont ignorées.
pub fn trace_best_path(
    domain: &Domain,
    sources: &[usize],
    targets: &[usize],
    min_cond: f32,
) -> Option<TracedPath> {
    let n = domain.num_nodes();
    let target_set: HashSet<usize> = targets.iter().copied().collect();

    let mut dist = vec![f64::INFINITY; n];
    let mut prev: Vec<Option<(usize, usize)>> = vec![None; n]; // (prev_node, edge_idx)
    let mut heap = BinaryHeap::new();

    for &src in sources {
        dist[src] = 0.0;
        heap.push(PathState { node: src, cost: 0.0 });
    }

    let mut reached_target: Option<usize> = None;

    while let Some(PathState { node, cost }) = heap.pop() {
        if cost > dist[node] {
            continue;
        }
        if target_set.contains(&node) {
            reached_target = Some(node);
            break;
        }

        // Explore outgoing edges
        let out_start = domain.outgoing.offsets[node];
        let out_end = domain.outgoing.offsets[node + 1];
        for k in out_start..out_end {
            let target_node = domain.outgoing.target_nodes[k];
            let edge_idx = domain.outgoing.edge_indices[k];
            let cond = domain.edges[edge_idx].conductance;
            if cond < min_cond {
                continue;
            }
            // Cost = log(C_max / cond) → higher conductance = lower cost
            // Utiliser C_max=5.0 (max config) pour garantir des coûts ≥ 0 (Dijkstra exige des poids positifs)
            let edge_cost = (5.0_f64 / (cond as f64)).ln();
            let new_dist = cost + edge_cost;
            if new_dist < dist[target_node] {
                dist[target_node] = new_dist;
                prev[target_node] = Some((node, edge_idx));
                heap.push(PathState { node: target_node, cost: new_dist });
            }
        }
    }

    reached_target.map(|target| {
        // Reconstruct path
        let mut nodes_path = vec![target];
        let mut edges_path = Vec::new();
        let mut current = target;
        while let Some((p, ei)) = prev[current] {
            nodes_path.push(p);
            edges_path.push(ei);
            current = p;
        }
        nodes_path.reverse();
        edges_path.reverse();

        let route_score: f64 = edges_path.iter()
            .map(|&ei| domain.edges[ei].conductance as f64)
            .sum();
        let min_conductance = edges_path.iter()
            .map(|&ei| domain.edges[ei].conductance as f64)
            .fold(f64::INFINITY, f64::min);

        TracedPath {
            nodes: nodes_path,
            edges: edges_path,
            route_score,
            min_conductance,
        }
    })
}

// =========================================================================
// RouteScore & Directional Index
// =========================================================================

/// Calcule le RouteScore entre un groupe d'entrée et un groupe de sortie.
/// RouteScore = somme des conductances sur le meilleur chemin.
pub fn route_score(
    domain: &Domain,
    input_group: &[usize],
    output_group: &[usize],
) -> f64 {
    match trace_best_path(domain, input_group, output_group, 0.05) {
        Some(path) => path.route_score,
        None => 0.0,
    }
}

/// Calcule l'indice directionnel D_k pour un input k :
/// D_k = RouteScore(I_k, O_target) - RouteScore(I_k, O_competitor)
pub fn directional_index(
    domain: &Domain,
    input_group: &[usize],
    target_output_group: &[usize],
    competitor_output_group: &[usize],
) -> f64 {
    let rs_target = route_score(domain, input_group, target_output_group);
    let rs_competitor = route_score(domain, input_group, competitor_output_group);
    rs_target - rs_competitor
}

// =========================================================================
// Cohérence topologique
// =========================================================================

/// Résultat d'un diagnostic complet.
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    /// RouteScores pour chaque paire (input_class → target_output)
    pub route_scores_target: Vec<f64>,
    /// RouteScores pour chaque paire (input_class → concurrent_output)
    pub route_scores_competitor: Vec<f64>,
    /// Indices directionnels D_k
    pub directional_indices: Vec<f64>,
    /// Chemins tracés (optionnel)
    pub traced_paths: Vec<Option<TracedPath>>,
    /// Cohérence topologique CT (si calculée)
    pub topological_coherence: Option<f64>,
    /// Sustain ratio (si calculé)
    pub sustain_ratio: Option<f64>,
}

/// Exécute un diagnostic complet pour une configuration I/O donnée.
pub fn run_diagnostics(
    domain: &Domain,
    input_groups: &[Vec<usize>],
    output_groups: &[Vec<usize>],
    target_mapping: &[usize],
    compute_ct: bool,
    initial_conductances: Option<&[f32]>,
) -> DiagnosticReport {
    let num_classes = input_groups.len();
    let mut route_scores_target = Vec::with_capacity(num_classes);
    let mut route_scores_competitor = Vec::with_capacity(num_classes);
    let mut directional_indices = Vec::with_capacity(num_classes);
    let mut traced_paths = Vec::with_capacity(num_classes);

    for k in 0..num_classes {
        let target_idx = target_mapping[k];
        let target_group = &output_groups[target_idx];

        // Trouve le meilleur concurrent (autre sortie)
        let competitor_idx = (0..output_groups.len())
            .find(|&i| i != target_idx)
            .unwrap_or(target_idx);
        let competitor_group = &output_groups[competitor_idx];

        let rs_t = route_score(domain, &input_groups[k], target_group);
        let rs_c = route_score(domain, &input_groups[k], competitor_group);

        route_scores_target.push(rs_t);
        route_scores_competitor.push(rs_c);
        directional_indices.push(rs_t - rs_c);

        let path = trace_best_path(domain, &input_groups[k], target_group, 0.05);
        traced_paths.push(path);
    }

    // Cohérence topologique CT = corrélation entre ΔC et on_useful_path
    let topological_coherence = if compute_ct {
        if let Some(initial_conds) = initial_conductances {
            Some(compute_topological_coherence(domain, initial_conds, &traced_paths))
        } else {
            None
        }
    } else {
        None
    };

    DiagnosticReport {
        route_scores_target,
        route_scores_competitor,
        directional_indices,
        traced_paths,
        topological_coherence,
        sustain_ratio: None, // Calculé séparément dans la boucle
    }
}

/// Calcule la cohérence topologique CT :
/// CT = corr(ΔC_ij, OnUsefulPath_ij)
fn compute_topological_coherence(
    domain: &Domain,
    initial_conductances: &[f32],
    paths: &[Option<TracedPath>],
) -> f64 {
    // Collecter les arêtes sur des chemins utiles
    let mut on_path: HashSet<usize> = HashSet::new();
    for path_opt in paths {
        if let Some(path) = path_opt {
            for &ei in &path.edges {
                on_path.insert(ei);
            }
        }
    }
    if on_path.is_empty() {
        return 0.0;
    }

    // Calculer les deux vecteurs pour la corrélation
    let n = domain.edges.len().min(initial_conductances.len());
    let mut delta_w = Vec::with_capacity(n);
    let mut is_useful = Vec::with_capacity(n);

    for i in 0..n {
        let dw = domain.edges[i].conductance as f64 - initial_conductances[i] as f64;
        delta_w.push(dw);
        is_useful.push(if on_path.contains(&i) { 1.0 } else { 0.0 });
    }

    pearson_correlation(&delta_w, &is_useful)
}

/// Corrélation de Pearson entre deux vecteurs.
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if n < 2.0 { return 0.0; }

    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-12 { 0.0 } else { cov / denom }
}

// =========================================================================
// Sustain Ratio
// =========================================================================

/// Tracker pour mesurer le sustain ratio au fil des ticks.
pub struct SustainTracker {
    /// Énergies collectées pendant les pics (période de stimulus)
    peak_energies: Vec<f64>,
    /// Énergies collectées pendant les inter-trials
    intertrial_energies: Vec<f64>,
    in_stimulus: bool,
}

impl SustainTracker {
    pub fn new() -> Self {
        SustainTracker {
            peak_energies: Vec::new(),
            intertrial_energies: Vec::new(),
            in_stimulus: false,
        }
    }

    /// Signale qu'on est en période de stimulus.
    pub fn set_in_stimulus(&mut self, in_stimulus: bool) {
        self.in_stimulus = in_stimulus;
    }

    /// Enregistre l'énergie globale du tick courant.
    pub fn record_energy(&mut self, energy: f64) {
        if self.in_stimulus {
            self.peak_energies.push(energy);
        } else {
            self.intertrial_energies.push(energy);
        }
    }

    /// Calcule le sustain ratio : mean_energy_intertrial / mean_energy_peak
    pub fn sustain_ratio(&self) -> f64 {
        let mean_peak = if self.peak_energies.is_empty() {
            1.0
        } else {
            self.peak_energies.iter().sum::<f64>() / self.peak_energies.len() as f64
        };
        let mean_inter = if self.intertrial_energies.is_empty() {
            0.0
        } else {
            self.intertrial_energies.iter().sum::<f64>() / self.intertrial_energies.len() as f64
        };
        if mean_peak < 1e-10 { 0.0 } else { mean_inter / mean_peak }
    }
}

// =========================================================================
// Printing
// =========================================================================

/// Affiche un rapport de diagnostics formaté.
pub fn print_diagnostic_report(report: &DiagnosticReport) {
    eprintln!("\n=== V5 Diagnostic Report ===");
    for (i, (rs_t, rs_c)) in report.route_scores_target.iter()
        .zip(report.route_scores_competitor.iter())
        .enumerate()
    {
        let d = report.directional_indices[i];
        let path_info = match &report.traced_paths[i] {
            Some(p) => format!("{} hops, bottleneck={:.4}", p.nodes.len() - 1, p.min_conductance),
            None => "no path found".to_string(),
        };
        eprintln!("  Class {} → target: RS={:.3}, competitor: RS={:.3}, D={:.3} ({})",
            i, rs_t, rs_c, d, path_info);
    }
    if let Some(ct) = report.topological_coherence {
        eprintln!("  Topological Coherence CT = {:.4}", ct);
    }
    if let Some(sr) = report.sustain_ratio {
        eprintln!("  Sustain Ratio = {:.4}", sr);
    }
    eprintln!();
}
