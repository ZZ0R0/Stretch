use serde::{Deserialize, Serialize};

use crate::domain::Domain;

/// V4 : Lecteur de sortie minimal.
///
/// Lit l'activité de groupes de nœuds de sortie et produit une décision
/// par winner-take-all (argmax des scores de groupe).
#[derive(Debug, Clone)]
pub struct OutputReader {
    /// Indices des nœuds de sortie, groupés par classe.
    /// groups[class_id] = vec de nœuds pour cette classe.
    pub groups: Vec<Vec<usize>>,
    /// Nombre de classes
    pub num_classes: usize,
}

/// Configuration de l'interface de sortie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Nombre de classes de sortie
    #[serde(default = "default_output_num_classes")]
    pub num_classes: usize,
    /// Nombre de nœuds par groupe de sortie
    #[serde(default = "default_output_group_size")]
    pub group_size: usize,
    /// Indice du premier nœud de sortie (groupes contigus)
    #[serde(default = "default_output_start_node")]
    pub start_node: usize,
    /// Nombre de ticks à attendre avant de lire la sortie (latence de traitement)
    #[serde(default = "default_output_read_delay")]
    pub read_delay: usize,
}

fn default_output_num_classes() -> usize { 2 }
fn default_output_group_size() -> usize { 50 }
fn default_output_start_node() -> usize { 500 }
fn default_output_read_delay() -> usize { 10 }

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            num_classes: default_output_num_classes(),
            group_size: default_output_group_size(),
            start_node: default_output_start_node(),
            read_delay: default_output_read_delay(),
        }
    }
}

/// Résultat d'un readout de sortie.
#[derive(Debug, Clone)]
pub struct ReadoutResult {
    /// Scores par classe (somme des activations du groupe)
    pub scores: Vec<f32>,
    /// Décision = argmax des scores
    pub decision: usize,
    /// Marge = score gagnant - second score
    pub margin: f32,
}

impl OutputReader {
    /// Construire le lecteur à partir de la config.
    pub fn new(config: &OutputConfig, domain: &Domain) -> Self {
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
        OutputReader {
            num_classes: config.num_classes,
            groups,
        }
    }

    /// Lire la sortie : calculer le score de chaque groupe et produire une décision.
    pub fn readout(&self, domain: &Domain) -> ReadoutResult {
        let mut scores: Vec<f32> = Vec::with_capacity(self.num_classes);
        for group in &self.groups {
            let sum: f32 = group.iter()
                .filter_map(|&idx| domain.nodes.get(idx))
                .map(|n| n.activation)
                .sum();
            scores.push(sum);
        }

        let mut best = 0;
        let mut best_score = f32::NEG_INFINITY;
        let mut second_score = f32::NEG_INFINITY;
        for (i, &s) in scores.iter().enumerate() {
            if s > best_score {
                second_score = best_score;
                best_score = s;
                best = i;
            } else if s > second_score {
                second_score = s;
            }
        }

        let margin = if second_score > f32::NEG_INFINITY {
            best_score - second_score
        } else {
            best_score
        };

        ReadoutResult {
            scores,
            decision: best,
            margin,
        }
    }
}
