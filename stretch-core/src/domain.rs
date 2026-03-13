use std::collections::HashMap;

use kiddo::{KdTree, SquaredEuclidean};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::{DomainConfig, EdgeDefaults, NodeDefaults};
use crate::edge::Edge;
use crate::node::Node;

/// Domaine spatial : graphe de nœuds et de liaisons.
/// V1 : positions 3D, indexation KD-tree, topologies KNN / radius / grille.
pub struct Domain {
    pub nodes: Vec<Node>,
    /// Liaisons indexées séquentiellement
    pub edges: Vec<Edge>,
    /// Index rapide : pour chaque nœud, liste des indices dans `edges` des liaisons sortantes
    pub adjacency: Vec<Vec<usize>>,
    /// Position 3D de chaque nœud
    pub positions: Vec<[f64; 3]>,
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

    // -----------------------------------------------------------------------
    // V0 compat : Grille 2D (side × side, z=0)
    // -----------------------------------------------------------------------
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

        Domain { nodes, edges, adjacency, positions }
    }

    // -----------------------------------------------------------------------
    // V0 compat : graphe sparse aléatoire 2D (z=0)
    // -----------------------------------------------------------------------
    fn build_random_sparse(
        n: usize,
        avg_neighbors: usize,
        seed: u64,
        node_defaults: &NodeDefaults,
        edge_defaults: &EdgeDefaults,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let nodes: Vec<Node> = (0..n).map(|i| Node::new(i, node_defaults)).collect();
        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut edge_set: HashMap<(usize, usize), bool> = HashMap::new();

        let positions: Vec<[f64; 3]> = (0..n)
            .map(|_| [rng.gen::<f64>() * 100.0, rng.gen::<f64>() * 100.0, 0.0])
            .collect();

        let p = (avg_neighbors as f64) / (n as f64 - 1.0).max(1.0);

        for i in 0..n {
            for j in (i + 1)..n {
                if rng.gen::<f64>() < p {
                    let dist = euclidean_3d(&positions[i], &positions[j]);

                    if !edge_set.contains_key(&(i, j)) {
                        edge_set.insert((i, j), true);
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

        Domain { nodes, edges, adjacency, positions }
    }

    // -----------------------------------------------------------------------
    // V1 : KNN graph 3D — K plus proches voisins (bidirectionnel)
    // -----------------------------------------------------------------------
    fn build_knn_3d(
        n: usize,
        k: usize,
        extent: f64,
        seed: u64,
        node_defaults: &NodeDefaults,
        edge_defaults: &EdgeDefaults,
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

        // Construire le KD-tree
        let mut tree: KdTree<f64, 3> = KdTree::new();
        for (i, pos) in positions.iter().enumerate() {
            tree.add(pos, i as u64);
        }

        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut edge_set: HashMap<(usize, usize), bool> = HashMap::new();

        // K+1 car le nœud se trouve lui-même dans les résultats
        let k_query = (k + 1).min(n);

        for i in 0..n {
            let neighbors = tree.nearest_n::<SquaredEuclidean>(&positions[i], k_query);
            for nb in &neighbors {
                let j = nb.item as usize;
                if j == i {
                    continue;
                }
                let canon = if i < j { (i, j) } else { (j, i) };
                if edge_set.contains_key(&canon) {
                    continue;
                }
                edge_set.insert(canon, true);
                let dist = euclidean_3d(&positions[i], &positions[j]);
                let idx_fwd = edges.len();
                edges.push(Edge::new(i, j, dist, edge_defaults));
                adjacency[i].push(idx_fwd);
                let idx_bwd = edges.len();
                edges.push(Edge::new(j, i, dist, edge_defaults));
                adjacency[j].push(idx_bwd);
            }
        }

        Domain { nodes, edges, adjacency, positions }
    }

    // -----------------------------------------------------------------------
    // V1 : Radius graph 3D — voisins dans un rayon spatial
    // -----------------------------------------------------------------------
    fn build_radius_3d(
        n: usize,
        radius: f64,
        extent: f64,
        seed: u64,
        node_defaults: &NodeDefaults,
        edge_defaults: &EdgeDefaults,
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
        let mut edge_set: HashMap<(usize, usize), bool> = HashMap::new();

        let radius_sq = radius * radius;

        for i in 0..n {
            let neighbors = tree.within::<SquaredEuclidean>(&positions[i], radius_sq);
            for nb in &neighbors {
                let j = nb.item as usize;
                if j == i {
                    continue;
                }
                let canon = if i < j { (i, j) } else { (j, i) };
                if edge_set.contains_key(&canon) {
                    continue;
                }
                edge_set.insert(canon, true);
                let dist = euclidean_3d(&positions[i], &positions[j]);
                let idx_fwd = edges.len();
                edges.push(Edge::new(i, j, dist, edge_defaults));
                adjacency[i].push(idx_fwd);
                let idx_bwd = edges.len();
                edges.push(Edge::new(j, i, dist, edge_defaults));
                adjacency[j].push(idx_bwd);
            }
        }

        Domain { nodes, edges, adjacency, positions }
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }
}

/// Distance euclidienne 3D.
fn euclidean_3d(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}
