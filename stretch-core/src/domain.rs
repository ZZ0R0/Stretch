use std::collections::HashMap;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::{DomainConfig, EdgeDefaults, NodeDefaults};
use crate::edge::Edge;
use crate::node::Node;

/// Domaine spatial : graphe de nœuds et de liaisons.
pub struct Domain {
    pub nodes: Vec<Node>,
    /// Liaisons indexées par (from, to) – clé canonique
    pub edges: Vec<Edge>,
    /// Index rapide : pour chaque nœud, liste des indices dans `edges` des liaisons sortantes
    pub adjacency: Vec<Vec<usize>>,
    /// Position 2D optionnelle (pour grilles)
    pub positions: Vec<(f64, f64)>,
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
            _ => panic!("Topologie inconnue : {}", config.topology),
        }
    }

    /// Construire une grille 2D de taille side × side avec voisinage 4-connexe.
    fn build_grid2d(side: usize, node_defaults: &NodeDefaults, edge_defaults: &EdgeDefaults) -> Self {
        let n = side * side;
        let nodes: Vec<Node> = (0..n).map(|i| Node::new(i, node_defaults)).collect();
        let mut edges = Vec::new();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut positions = Vec::with_capacity(n);

        for i in 0..n {
            let row = i / side;
            let col = i % side;
            positions.push((col as f64, row as f64));
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

        Domain {
            nodes,
            edges,
            adjacency,
            positions,
        }
    }

    /// Construire un graphe sparse aléatoire.
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

        // Positions aléatoires dans [0, 100]²
        let positions: Vec<(f64, f64)> = (0..n)
            .map(|_| (rng.gen::<f64>() * 100.0, rng.gen::<f64>() * 100.0))
            .collect();

        // Probabilité de connexion pour obtenir avg_neighbors voisins en moyenne
        let p = (avg_neighbors as f64) / (n as f64 - 1.0).max(1.0);

        for i in 0..n {
            for j in (i + 1)..n {
                if rng.gen::<f64>() < p {
                    let dx = positions[i].0 - positions[j].0;
                    let dy = positions[i].1 - positions[j].1;
                    let dist = (dx * dx + dy * dy).sqrt();

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

        Domain {
            nodes,
            edges,
            adjacency,
            positions,
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }
}
