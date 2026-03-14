use std::collections::HashSet;

use kiddo::{KdTree, SquaredEuclidean};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::{DomainConfig, EdgeDefaults, NodeDefaults, PropagationConfig};
use crate::edge::Edge;
use crate::node::{Node, NeuronType};

/// CSR (Compressed Sparse Row) pour les arêtes entrantes par nœud cible.
/// Stockage contigu en mémoire — élimine les 50k allocations heap de Vec<Vec<usize>>.
pub struct IncomingCSR {
    /// offsets[i]..offsets[i+1] = range des entrées pour le nœud i
    pub offsets: Vec<usize>,
    /// Nœud source pour chaque arête entrante
    pub source_nodes: Vec<usize>,
    /// Index dans Domain.edges pour chaque arête entrante
    pub edge_indices: Vec<usize>,
    /// Poids du kernel spatial pré-calculé (exp(-λ*d)), rempli par set_kernel_weights
    pub kernel_weights: Vec<f32>,
}

impl IncomingCSR {
    fn empty() -> Self {
        IncomingCSR {
            offsets: vec![0],
            source_nodes: Vec::new(),
            edge_indices: Vec::new(),
            kernel_weights: Vec::new(),
        }
    }
}

/// CSR sortant pour la propagation fired-only (C1).
/// Indexé par nœud source — pour chaque source, liste des arêtes sortantes.
pub struct OutgoingCSR {
    /// offsets[i]..offsets[i+1] = range des arêtes sortantes pour le nœud i
    pub offsets: Vec<usize>,
    /// Nœud cible pour chaque arête sortante
    pub target_nodes: Vec<usize>,
    /// Index dans Domain.edges pour chaque arête sortante
    pub edge_indices: Vec<usize>,
    /// Poids du kernel spatial pré-calculé pour chaque arête sortante
    pub kernel_weights: Vec<f32>,
}

impl OutgoingCSR {
    fn empty() -> Self {
        OutgoingCSR {
            offsets: vec![0],
            target_nodes: Vec::new(),
            edge_indices: Vec::new(),
            kernel_weights: Vec::new(),
        }
    }
}

/// Domaine spatial : graphe de nœuds et de liaisons.
pub struct Domain {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// Original edge count (preserved after compact_for_gpu drops edges)
    edge_count: usize,
    /// Index rapide : pour chaque nœud, indices des arêtes sortantes (pour budget synaptique)
    pub adjacency: Vec<Vec<usize>>,
    /// CSR des arêtes entrantes par nœud cible (pour propagation)
    pub incoming: IncomingCSR,
    /// CSR des arêtes sortantes par nœud source (C1: fired-only propagation)
    pub outgoing: OutgoingCSR,
    /// Position 3D de chaque nœud
    pub positions: Vec<[f64; 3]>,
    /// Conductances extraites (parallèle à edges) — cache pour propagation
    pub conductances: Vec<f32>,
    /// Bitmap : true si le nœud est excitateur
    pub node_is_excitatory: Vec<bool>,
    /// KD-tree persisté pour les requêtes spatiales
    pub kdtree: Option<KdTree<f64, 3>>,
    /// C3: indices des arêtes "chaudes" (|eligibility| > ε)
    pub hot_edge_indices: Vec<usize>,
    /// C3: bitmap — arête est-elle dans hot_edge_indices ?
    pub edge_is_hot: Vec<bool>,
    /// C4: budget synaptique courant par nœud source (running totals)
    pub running_totals: Vec<f32>,
    /// C5: bitmap — le nœud a besoin d'une mise à jour dissipation ?
    pub node_needs_update: Vec<bool>,
}

impl Domain {
    pub fn from_config(
        config: &DomainConfig,
        node_defaults: &NodeDefaults,
        edge_defaults: &EdgeDefaults,
    ) -> Self {
        match config.topology.as_str() {
            "grid2d" => Self::build_grid2d(config.size, node_defaults, edge_defaults),
            "random_sparse" => Self::build_random_sparse(
                config.size,
                config.avg_neighbors,
                config.seed,
                node_defaults,
                edge_defaults,
            ),
            "knn_3d" => Self::build_knn_3d(
                config.size,
                config.k_neighbors,
                config.domain_extent,
                config.seed,
                node_defaults,
                edge_defaults,
            ),
            "radius_3d" => Self::build_radius_3d(
                config.size,
                config.radius,
                config.domain_extent,
                config.seed,
                node_defaults,
                edge_defaults,
            ),
            _ => panic!("Topologie inconnue : {}", config.topology),
        }
    }

    /// Finaliser le domaine après construction : CSR, conductances, bitmap.
    fn finalize(&mut self) {
        self.build_incoming_csr();
        self.build_outgoing_csr();
        self.sync_conductances();
        self.init_node_flags();
        self.init_optimization_structures();
    }

    fn build_grid2d(side: usize, node_defaults: &NodeDefaults, edge_defaults: &EdgeDefaults) -> Self {
        let n = side * side;
        let nodes: Vec<Node> = (0..n).map(|i| Node::new(i, node_defaults)).collect();
        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut positions = Vec::with_capacity(n);

        for i in 0..n {
            let row = i / side;
            let col = i % side;
            positions.push([col as f64, row as f64, 0.0]);
        }

        let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for i in 0..n {
            let row = (i / side) as i32;
            let col = (i % side) as i32;
            for &(dr, dc) in &directions {
                let nr = row + dr;
                let nc = col + dc;
                if nr >= 0 && nr < side as i32 && nc >= 0 && nc < side as i32 {
                    let j = (nr as usize) * side + nc as usize;
                    let edge_idx = edges.len();
                    edges.push(Edge::new(i, j, 1.0, edge_defaults));
                    adjacency[i].push(edge_idx);
                }
            }
        }

        let n = nodes.len();
        let ne = edges.len();
        let mut d = Domain {
            nodes, edges, edge_count: ne, adjacency, incoming: IncomingCSR::empty(),
            outgoing: OutgoingCSR::empty(),
            positions, conductances: Vec::new(), node_is_excitatory: Vec::new(), kdtree: None,
            hot_edge_indices: Vec::new(), edge_is_hot: vec![false; ne],
            running_totals: vec![0.0; n], node_needs_update: vec![true; n],
        };
        d.finalize();
        d
    }

    fn build_random_sparse(
        n: usize, avg_neighbors: usize, seed: u64,
        node_defaults: &NodeDefaults, edge_defaults: &EdgeDefaults,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let nodes: Vec<Node> = (0..n).map(|i| Node::new(i, node_defaults)).collect();
        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut edge_set: HashSet<(usize, usize)> = HashSet::new();

        let positions: Vec<[f64; 3]> = (0..n)
            .map(|_| [rng.gen::<f64>() * 100.0, rng.gen::<f64>() * 100.0, 0.0])
            .collect();

        let p = (avg_neighbors as f64) / (n as f64 - 1.0).max(1.0);
        for i in 0..n {
            for j in (i + 1)..n {
                if rng.gen::<f64>() < p {
                    let dist = euclidean_3d(&positions[i], &positions[j]);
                    if edge_set.insert((i, j)) {
                        let idx_fwd = edges.len();
                        edges.push(Edge::new(i, j, dist, edge_defaults));
                        adjacency[i].push(idx_fwd);
                        let idx_bwd = edges.len();
                        edges.push(Edge::new(j, i, dist, edge_defaults));
                        adjacency[j].push(idx_bwd);
                    }
                }
            }
        }

        let nn = nodes.len();
        let ne = edges.len();
        let mut d = Domain {
            nodes, edges, edge_count: ne, adjacency, incoming: IncomingCSR::empty(),
            outgoing: OutgoingCSR::empty(),
            positions, conductances: Vec::new(), node_is_excitatory: Vec::new(), kdtree: None,
            hot_edge_indices: Vec::new(), edge_is_hot: vec![false; ne],
            running_totals: vec![0.0; nn], node_needs_update: vec![true; nn],
        };
        d.finalize();
        d
    }

    fn build_knn_3d(
        n: usize, k: usize, extent: f64, seed: u64,
        node_defaults: &NodeDefaults, edge_defaults: &EdgeDefaults,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let nodes: Vec<Node> = (0..n).map(|i| Node::new(i, node_defaults)).collect();
        let positions: Vec<[f64; 3]> = (0..n)
            .map(|_| [
                rng.gen::<f64>() * extent,
                rng.gen::<f64>() * extent,
                rng.gen::<f64>() * extent,
            ])
            .collect();

        let mut tree: KdTree<f64, 3> = KdTree::new();
        for (i, pos) in positions.iter().enumerate() {
            tree.add(pos, i as u64);
        }

        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        {
            let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
            let k_query = (k + 1).min(n);
            for i in 0..n {
                let neighbors = tree.nearest_n::<SquaredEuclidean>(&positions[i], k_query);
                for nb in &neighbors {
                    let j = nb.item as usize;
                    if j == i { continue; }
                    let canon = if i < j { (i, j) } else { (j, i) };
                    if !edge_set.insert(canon) { continue; }
                    let dist = euclidean_3d(&positions[i], &positions[j]);
                    let idx_fwd = edges.len();
                    edges.push(Edge::new(i, j, dist, edge_defaults));
                    adjacency[i].push(idx_fwd);
                    let idx_bwd = edges.len();
                    edges.push(Edge::new(j, i, dist, edge_defaults));
                    adjacency[j].push(idx_bwd);
                }
            }
        } // edge_set dropped here — frees ~1.5 GB at 5M nodes

        let n = nodes.len();
        let ne = edges.len();
        let mut d = Domain {
            nodes, edges, edge_count: ne, adjacency, incoming: IncomingCSR::empty(),
            outgoing: OutgoingCSR::empty(),
            positions, conductances: Vec::new(), node_is_excitatory: Vec::new(),
            kdtree: Some(tree),
            hot_edge_indices: Vec::new(), edge_is_hot: vec![false; ne],
            running_totals: vec![0.0; n], node_needs_update: vec![true; n],
        };
        d.finalize();
        d
    }

    fn build_radius_3d(
        n: usize, radius: f64, extent: f64, seed: u64,
        node_defaults: &NodeDefaults, edge_defaults: &EdgeDefaults,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let nodes: Vec<Node> = (0..n).map(|i| Node::new(i, node_defaults)).collect();
        let positions: Vec<[f64; 3]> = (0..n)
            .map(|_| [
                rng.gen::<f64>() * extent,
                rng.gen::<f64>() * extent,
                rng.gen::<f64>() * extent,
            ])
            .collect();

        let mut tree: KdTree<f64, 3> = KdTree::new();
        for (i, pos) in positions.iter().enumerate() {
            tree.add(pos, i as u64);
        }

        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let radius_sq = radius * radius;
        {
            let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
            for i in 0..n {
                let neighbors = tree.within::<SquaredEuclidean>(&positions[i], radius_sq);
                for nb in &neighbors {
                    let j = nb.item as usize;
                    if j == i { continue; }
                    let canon = if i < j { (i, j) } else { (j, i) };
                    if !edge_set.insert(canon) { continue; }
                    let dist = euclidean_3d(&positions[i], &positions[j]);
                    let idx_fwd = edges.len();
                    edges.push(Edge::new(i, j, dist, edge_defaults));
                    adjacency[i].push(idx_fwd);
                    let idx_bwd = edges.len();
                    edges.push(Edge::new(j, i, dist, edge_defaults));
                    adjacency[j].push(idx_bwd);
                }
            }
        } // edge_set dropped here

        let n = nodes.len();
        let ne = edges.len();
        let mut d = Domain {
            nodes, edges, edge_count: ne, adjacency, incoming: IncomingCSR::empty(),
            outgoing: OutgoingCSR::empty(),
            positions, conductances: Vec::new(), node_is_excitatory: Vec::new(),
            kdtree: Some(tree),
            hot_edge_indices: Vec::new(), edge_is_hot: vec![false; ne],
            running_totals: vec![0.0; n], node_needs_update: vec![true; n],
        };
        d.finalize();
        d
    }

    pub fn num_nodes(&self) -> usize { self.nodes.len() }
    pub fn num_edges(&self) -> usize { self.edge_count }

    /// Drop CPU-only structures to free memory when running on GPU.
    /// Keeps: nodes (for viz sync), edges (for end-of-sim sync), positions, incoming CSR, node_is_excitatory.
    /// Drops: adjacency, outgoing CSR, kdtree, conductances, hot_edge_indices, edge_is_hot, running_totals, node_needs_update.
    pub fn compact_for_gpu(&mut self) {
        // edges: 57M × 56 = 3.2 GB at 5M nodes — rebuilt from GPU at sync_state_to_domain
        let n_edges = self.edges.len();
        self.edges = Vec::new();
        // adjacency: 5M Vecs = ~760 MB at 5M nodes
        self.adjacency = Vec::new();
        // outgoing CSR: ~1.18 GB at 5M nodes
        self.outgoing = OutgoingCSR::empty();
        // incoming CSR: ~1.18 GB at 5M nodes (already uploaded to GPU)
        self.incoming = IncomingCSR::empty();
        // kdtree: ~300 MB at 5M nodes
        self.kdtree = None;
        // conductances: 57M × 4 = 228 MB at 5M nodes
        self.conductances = Vec::new();
        // CPU optimization structures
        self.hot_edge_indices = Vec::new();
        self.edge_is_hot = Vec::new();
        self.running_totals = Vec::new();
        self.node_needs_update = Vec::new();

        let saved_mb = n_edges * 56 / (1024 * 1024); // edges is the biggest saving
        eprintln!("[GPU] Compacted domain: freed ~{} MB of edges + adjacency, CSR, kdtree, conductances, caches", saved_mb);
    }

    /// Drop the KD-tree to free memory (called after spatial setup is done).
    pub fn drop_kdtree(&mut self) {
        self.kdtree = None;
    }

    /// Construire le CSR des arêtes entrantes.
    fn build_incoming_csr(&mut self) {
        let n = self.nodes.len();
        // Temporairement construire Vec<Vec> puis convertir en CSR
        let mut incoming: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (idx, edge) in self.edges.iter().enumerate() {
            incoming[edge.to].push(idx);
        }
        let total: usize = incoming.iter().map(|v| v.len()).sum();
        let mut offsets = Vec::with_capacity(n + 1);
        let mut source_nodes = Vec::with_capacity(total);
        let mut edge_indices = Vec::with_capacity(total);
        let mut offset = 0;
        for adj in &incoming {
            offsets.push(offset);
            for &edge_idx in adj {
                source_nodes.push(self.edges[edge_idx].from);
                edge_indices.push(edge_idx);
            }
            offset += adj.len();
        }
        offsets.push(offset);
        self.incoming = IncomingCSR {
            offsets, source_nodes, edge_indices,
            kernel_weights: Vec::new(),
        };
    }

    /// Construire le CSR des arêtes sortantes (C1: fired-only propagation).
    fn build_outgoing_csr(&mut self) {
        let n = self.nodes.len();
        let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (idx, edge) in self.edges.iter().enumerate() {
            outgoing[edge.from].push(idx);
        }
        let total: usize = outgoing.iter().map(|v| v.len()).sum();
        let mut offsets = Vec::with_capacity(n + 1);
        let mut target_nodes = Vec::with_capacity(total);
        let mut edge_indices = Vec::with_capacity(total);
        let mut offset = 0;
        for adj in &outgoing {
            offsets.push(offset);
            for &edge_idx in adj {
                target_nodes.push(self.edges[edge_idx].to);
                edge_indices.push(edge_idx);
            }
            offset += adj.len();
        }
        offsets.push(offset);
        self.outgoing = OutgoingCSR {
            offsets, target_nodes, edge_indices,
            kernel_weights: Vec::new(),
        };
    }

    /// Initialiser les structures d'optimisation (C3/C4/C5).
    fn init_optimization_structures(&mut self) {
        let n = self.nodes.len();
        let ne = self.edges.len();
        self.hot_edge_indices = Vec::new();
        self.edge_is_hot = vec![false; ne];
        // C4: initialiser les running totals = somme des conductances par nœud source
        self.running_totals = vec![0.0; n];
        for edge in &self.edges {
            self.running_totals[edge.from] += edge.conductance;
        }
        self.node_needs_update = vec![true; n];
    }

    /// Initialiser les kernel weights dans le CSR sortant (appelé avec la config propagation).
    pub fn set_outgoing_kernel_weights(&mut self, config: &PropagationConfig) {
        let is_exp = config.kernel == "exponential";
        let sd = config.spatial_decay;
        self.outgoing.kernel_weights = self.outgoing.edge_indices.iter().map(|&idx| {
            let d = self.edges[idx].distance as f64;
            if is_exp { (-sd * d).exp() as f32 } else { (-0.5 * (d * sd).powi(2)).exp() as f32 }
        }).collect();
    }

    /// Initialiser les kernel weights dans les CSR (appelé après construction avec la config propagation).
    pub fn set_incoming_kernel_weights(&mut self, config: &PropagationConfig) {
        let is_exp = config.kernel == "exponential";
        let sd = config.spatial_decay;
        self.incoming.kernel_weights = self.incoming.edge_indices.iter().map(|&idx| {
            let d = self.edges[idx].distance as f64;
            if is_exp { (-sd * d).exp() as f32 } else { (-0.5 * (d * sd).powi(2)).exp() as f32 }
        }).collect();
        self.set_outgoing_kernel_weights(config);
    }

    /// Synchroniser le cache de conductances depuis les arêtes (parallel).
    pub fn sync_conductances(&mut self) {
        use rayon::prelude::*;
        self.conductances.resize(self.edges.len(), 0.0);
        self.conductances.par_iter_mut()
            .zip(self.edges.par_iter())
            .for_each(|(c, edge)| {
                *c = edge.conductance;
            });
    }

    /// Initialiser les flags par nœud (excitatory bitmap).
    fn init_node_flags(&mut self) {
        self.node_is_excitatory = self.nodes.iter()
            .map(|n| n.node_type == NeuronType::Excitatory)
            .collect();
    }

    /// Assigner les types E/I et mettre à jour le bitmap.
    pub fn assign_neuron_types(&mut self, fraction: f64, seed: u64) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(77777));
        for node in &mut self.nodes {
            if rng.gen::<f64>() < fraction {
                node.node_type = NeuronType::Inhibitory;
            }
        }
        self.init_node_flags();
    }

    /// Sélectionner les N nœuds les plus proches d'un point 3D (utilise KD-tree si disponible).
    pub fn select_nearest_nodes(&self, center: [f64; 3], count: usize, exclude: &[usize]) -> Vec<usize> {
        if let Some(ref tree) = self.kdtree {
            // KD-tree : O(k log n) au lieu de O(n)
            let exclude_set: std::collections::HashSet<usize> = exclude.iter().copied().collect();
            let query_count = (count + exclude.len() + 10).min(self.nodes.len());
            let neighbors = tree.nearest_n::<SquaredEuclidean>(&center, query_count);
            let mut result = Vec::with_capacity(count);
            for nb in &neighbors {
                let idx = nb.item as usize;
                if !exclude_set.contains(&idx) {
                    result.push(idx);
                    if result.len() >= count { break; }
                }
            }
            result
        } else {
            // Fallback : scan linéaire O(n)
            let exclude_set: std::collections::HashSet<usize> = exclude.iter().copied().collect();
            let mut distances: Vec<(usize, f64)> = self.positions.iter()
                .enumerate()
                .filter(|(i, _)| !exclude_set.contains(i))
                .map(|(i, p)| (i, euclidean_3d(p, &center)))
                .collect();
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            distances.iter().take(count).map(|(i, _)| *i).collect()
        }
    }
}

fn euclidean_3d(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}
