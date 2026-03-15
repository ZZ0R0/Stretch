//! V5 : Système de tâches anti-biais topologique.
//!
//! Gère le placement I/O selon différentes stratégies pour prouver
//! que l'apprentissage ne dépend pas de la géométrie seule.

use crate::config::{V5TaskMode, V5BaselineMode};
use crate::domain::Domain;
use crate::simulation::Trial;

/// Résultat du placement I/O par le TaskSystem.
pub struct IoPlacement {
    pub input_groups: Vec<Vec<usize>>,
    pub output_groups: Vec<Vec<usize>>,
    /// Mapping classe_input -> classe_output (pour inverted/remap)
    pub target_mapping: Vec<usize>,
}

/// Place les groupes I/O selon le mode de tâche V5.
///
/// # Modes
/// - **Legacy** : V4 classique (input-0 à gauche, output-0 à 25%, input-1 à droite, output-1 à 75%)
/// - **Symmetric** : tous les groupes au centre du cube, à distances comparables
/// - **Inverted** : même géométrie que Legacy mais mapping inversé (input-0 → output-1)
/// - **Remap** : commence en Legacy, inversera plus tard
pub fn place_io(
    domain: &Domain,
    task_mode: &V5TaskMode,
    num_classes: usize,
    group_size: usize,
    extent: f64,
) -> IoPlacement {
    let mid = extent / 2.0;

    match task_mode {
        V5TaskMode::Legacy | V5TaskMode::Remap => {
            // V4-compatible : asymétrie spatiale forte
            let mut used: Vec<usize> = Vec::new();
            let in0 = domain.select_nearest_nodes([0.0, mid, mid], group_size, &used);
            used.extend(&in0);
            let in1 = domain.select_nearest_nodes([extent, mid, mid], group_size, &used);
            used.extend(&in1);
            let out0 = domain.select_nearest_nodes([extent * 0.25, mid, mid], group_size, &used);
            used.extend(&out0);
            let out1 = domain.select_nearest_nodes([extent * 0.75, mid, mid], group_size, &used);

            let mapping = (0..num_classes).collect();
            IoPlacement {
                input_groups: vec![in0, in1],
                output_groups: vec![out0, out1],
                target_mapping: mapping,
            }
        }

        V5TaskMode::Symmetric => {
            // Symétrique : les 4 groupes sont placés en croix autour du centre,
            // à la MÊME distance, mais sur des axes perpendiculaires.
            // Input-0 et Input-1 sur l'axe Y, Output-0 et Output-1 sur l'axe Z.
            // Ainsi chaque input est à distance égale des deux outputs.
            let offset = extent * 0.2; // distance du centre

            let mut used: Vec<usize> = Vec::new();
            // Inputs sur l'axe Y
            let in0 = domain.select_nearest_nodes([mid, mid - offset, mid], group_size, &used);
            used.extend(&in0);
            let in1 = domain.select_nearest_nodes([mid, mid + offset, mid], group_size, &used);
            used.extend(&in1);
            // Outputs sur l'axe Z
            let out0 = domain.select_nearest_nodes([mid, mid, mid - offset], group_size, &used);
            used.extend(&out0);
            let out1 = domain.select_nearest_nodes([mid, mid, mid + offset], group_size, &used);

            let mapping = (0..num_classes).collect();
            IoPlacement {
                input_groups: vec![in0, in1],
                output_groups: vec![out0, out1],
                target_mapping: mapping,
            }
        }

        V5TaskMode::Inverted => {
            // Même placement géométrique que Legacy (input-0 proche de output-0),
            // mais le mapping est INVERSÉ : input-0 doit activer output-1.
            // Si le réseau "apprend", il doit re-router contre la géométrie.
            let mut used: Vec<usize> = Vec::new();
            let in0 = domain.select_nearest_nodes([0.0, mid, mid], group_size, &used);
            used.extend(&in0);
            let in1 = domain.select_nearest_nodes([extent, mid, mid], group_size, &used);
            used.extend(&in1);
            let out0 = domain.select_nearest_nodes([extent * 0.25, mid, mid], group_size, &used);
            used.extend(&out0);
            let out1 = domain.select_nearest_nodes([extent * 0.75, mid, mid], group_size, &used);

            // Mapping inversé : classe 0 → output 1, classe 1 → output 0
            let mapping = vec![1, 0];
            IoPlacement {
                input_groups: vec![in0, in1],
                output_groups: vec![out0, out1],
                target_mapping: mapping,
            }
        }
    }
}

/// Génère les trials V5 avec support du mapping et de la présentation configurable.
pub fn generate_trials(
    num_classes: usize,
    target_mapping: &[usize],
    total_ticks: usize,
    presentation_ticks: usize,
    read_delay: usize,
    warmup_ticks: usize,
    inter_trial_gap: usize,
) -> Vec<Trial> {
    let trial_period = presentation_ticks + read_delay + inter_trial_gap + 1;
    let max_trials = if total_ticks > 0 { total_ticks / trial_period } else { 200 };

    let mut trials = Vec::with_capacity(max_trials);
    for i in 0..max_trials {
        let input_class = i % num_classes;
        let start_tick = warmup_ticks + i * trial_period;
        if total_ticks > 0 && start_tick + presentation_ticks + read_delay >= total_ticks {
            break;
        }
        trials.push(Trial {
            class: input_class,
            start_tick,
            presentation_ticks,
            read_delay,
            target_class: target_mapping[input_class],
        });
    }
    trials
}

/// Effectue le remap à mi-parcours (pour V5TaskMode::Remap).
/// Retourne les nouveaux trials à partir du tick donné.
pub fn remap_trials(
    trials: &mut Vec<Trial>,
    remap_at_tick: usize,
    new_mapping: &[usize],
) {
    for trial in trials.iter_mut() {
        if trial.start_tick >= remap_at_tick {
            trial.target_class = new_mapping[trial.class];
        }
    }
}

/// Applique le mode baseline : modifie les paramètres du domaine.
pub fn apply_baseline_mode(domain: &mut Domain, mode: &V5BaselineMode, seed: u64) {
    match mode {
        V5BaselineMode::FullLearning => {
            // Rien à modifier
        }
        V5BaselineMode::TopologyOnly => {
            // Rien ici — la plasticité sera désactivée dans la boucle de simulation
        }
        V5BaselineMode::RandomBaseline => {
            // Randomiser les conductances
            use rand::SeedableRng;
            use rand::Rng;
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            for edge in domain.edges.iter_mut() {
                edge.conductance = rng.gen_range(0.1..5.0_f32);
            }
            domain.sync_conductances();
        }
    }
}
