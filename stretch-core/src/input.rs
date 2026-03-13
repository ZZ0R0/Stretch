use serde::{Deserialize, Serialize};

use crate::domain::Domain;

/// V4 : Encodeur d'entrée minimal.
///
/// Projette un pattern discret (classe 0, 1, 2...) sur un sous-graphe d'entrée.
/// Chaque pattern est encodé comme une activation sparse sur un groupe de nœuds.
#[derive(Debug, Clone)]
pub struct InputEncoder {
    /// Indices des nœuds d'entrée, groupés par pattern.
    /// groups[pattern_id] = vec de nœuds pour ce pattern.
    pub groups: Vec<Vec<usize>>,
    /// Nombre de classes
    pub num_classes: usize,
}

/// Configuration de l'interface d'entrée.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Nombre de classes de patterns
    #[serde(default = "default_input_num_classes")]
    pub num_classes: usize,
    /// Nombre de nœuds par groupe d'entrée
    #[serde(default = "default_input_group_size")]
    pub group_size: usize,
    /// Indice du premier nœud d'entrée (les groupes sont contigus)
    #[serde(default)]
    pub start_node: usize,
    /// Intensité d'injection par nœud d'entrée
    #[serde(default = "default_input_intensity")]
    pub intensity: f64,
}

fn default_input_num_classes() -> usize { 2 }
fn default_input_group_size() -> usize { 50 }
fn default_input_intensity() -> f64 { 1.5 }

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig {
            num_classes: default_input_num_classes(),
            group_size: default_input_group_size(),
            start_node: 0,
            intensity: default_input_intensity(),
        }
    }
}

impl InputEncoder {
    /// Construire l'encodeur à partir de la config.
    /// Les groupes sont des tranches contigues de nœuds à partir de start_node.
    pub fn new(config: &InputConfig, domain: &Domain) -> Self {
        let n = domain.num_nodes();
        let mut groups = Vec::with_capacity(config.num_classes);
        for class in 0..config.num_classes {
            let base = config.start_node + class * config.group_size;
            let end = (base + config.group_size).min(n);
            if base < n {
                groups.push((base..end).collect());
            } else {
                groups.push(Vec::new());
            }
        }
        InputEncoder {
            num_classes: config.num_classes,
            groups,
        }
    }

    /// Injecter un pattern (classe) dans le domaine.
    /// Active les nœuds du groupe correspondant avec l'intensité donnée.
    pub fn inject(&self, domain: &mut Domain, class: usize, intensity: f64) {
        if class >= self.groups.len() {
            return;
        }
        for &node_idx in &self.groups[class] {
            if node_idx < domain.nodes.len() {
                domain.nodes[node_idx].inject_stimulus(intensity);
            }
        }
    }

    /// Tous les nœuds d'entrée (union de tous les groupes).
    pub fn all_input_nodes(&self) -> Vec<usize> {
        self.groups.iter().flat_map(|g| g.iter().copied()).collect()
    }
}
