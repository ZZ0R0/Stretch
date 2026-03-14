use serde::{Deserialize, Serialize};

use crate::config::NodeDefaults;

/// V3 : Type de neurone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuronType {
    Excitatory,
    Inhibitory,
}

/// État complet d'un nœud du système.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    /// Niveau d'activité courant [0, ∞) borné en pratique
    pub activation: f32,
    /// Seuil de déclenchement
    pub threshold: f32,
    /// Fatigue / réfractarité courante
    pub fatigue: f32,
    /// Trace mémoire locale cumulée
    pub memory_trace: f32,
    /// Facilité à répondre (multiplicateur)
    pub excitability: f32,
    /// Résistance locale à l'activation
    pub inhibition: f32,
    /// Compteur d'activations (instrumentation)
    pub activation_count: u64,
    /// V3 : type de neurone (E/I)
    pub node_type: NeuronType,
    /// V3 : dernier tick où le nœud était actif (pour STDP)
    pub last_activation_tick: Option<usize>,
    /// V3 : modulation du seuil par le PID indirect de zone
    pub threshold_mod: f32,
}

impl Node {
    pub fn new(id: usize, defaults: &NodeDefaults) -> Self {
        Node {
            id,
            activation: defaults.activation as f32,
            threshold: defaults.threshold as f32,
            fatigue: defaults.fatigue as f32,
            memory_trace: defaults.memory_trace as f32,
            excitability: defaults.excitability as f32,
            inhibition: defaults.inhibition as f32,
            activation_count: 0,
            node_type: NeuronType::Excitatory,
            last_activation_tick: None,
            threshold_mod: 0.0,
        }
    }

    /// Seuil effectif corrigé par fatigue, excitabilité, inhibition et modulation de zone.
    /// Borné inférieurement à 0.01 pour éviter que le PID rende tous les nœuds actifs.
    pub fn effective_threshold(&self) -> f32 {
        let raw = (self.threshold + self.fatigue + self.inhibition + self.threshold_mod) / self.excitability.max(0.01);
        raw.max(0.05)
    }

    /// Le nœud est-il considéré comme "actif" ?
    pub fn is_active(&self) -> bool {
        self.activation > self.effective_threshold()
    }

    /// Appliquer la dissipation de l'activation avec potentiel de repos.
    /// `rate` : taux de decay (déjà jitté si applicable)
    /// `min`  : activation minimale (potentiel de repos)
    pub fn decay_activation(&mut self, rate: f32, min: f32) {
        self.activation *= 1.0 - rate;
        self.activation = self.activation.max(min);
    }

    /// Mettre à jour la fatigue.
    pub fn update_fatigue(&mut self, gain: f32, recovery: f32) {
        if self.is_active() {
            self.fatigue += gain * self.activation;
        }
        self.fatigue *= 1.0 - recovery;
        self.fatigue = self.fatigue.clamp(0.0, 10.0);
    }

    /// Mettre à jour l'inhibition.
    pub fn update_inhibition(&mut self, gain: f32, decay: f32) {
        if self.is_active() {
            self.inhibition += gain;
        }
        self.inhibition *= 1.0 - decay;
        self.inhibition = self.inhibition.clamp(0.0, 10.0);
    }

    /// Mettre à jour la trace mémoire locale.
    pub fn update_trace(&mut self, trace_gain: f32, trace_decay: f32) {
        if self.is_active() {
            self.memory_trace += trace_gain * self.activation;
            self.activation_count += 1;
        }
        self.memory_trace *= 1.0 - trace_decay;
        self.memory_trace = self.memory_trace.clamp(0.0, 100.0);
    }

    /// Influence de la trace mémoire sur l'excitabilité :
    /// plus un nœud a été traversé, plus il est facile à réactiver.
    pub fn update_excitability_from_trace(&mut self) {
        self.excitability = 1.0 + 0.1 * self.memory_trace.min(5.0);
    }

    /// Injecter un stimulus externe.
    pub fn inject_stimulus(&mut self, intensity: f32) {
        self.activation += intensity;
    }
}
