use serde::{Deserialize, Serialize};

use crate::config::NodeDefaults;

/// État complet d'un nœud du système.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    /// Niveau d'activité courant [0, ∞) borné en pratique
    pub activation: f64,
    /// Seuil de déclenchement
    pub threshold: f64,
    /// Fatigue / réfractarité courante
    pub fatigue: f64,
    /// Trace mémoire locale cumulée
    pub memory_trace: f64,
    /// Facilité à répondre (multiplicateur)
    pub excitability: f64,
    /// Résistance locale à l'activation
    pub inhibition: f64,
    /// Compteur d'activations (instrumentation)
    pub activation_count: u64,
}

impl Node {
    pub fn new(id: usize, defaults: &NodeDefaults) -> Self {
        Node {
            id,
            activation: defaults.activation,
            threshold: defaults.threshold,
            fatigue: defaults.fatigue,
            memory_trace: defaults.memory_trace,
            excitability: defaults.excitability,
            inhibition: defaults.inhibition,
            activation_count: 0,
        }
    }

    /// Seuil effectif corrigé par fatigue, excitabilité et inhibition.
    pub fn effective_threshold(&self) -> f64 {
        (self.threshold + self.fatigue + self.inhibition) / self.excitability.max(0.01)
    }

    /// Le nœud est-il considéré comme "actif" ?
    pub fn is_active(&self) -> bool {
        self.activation > self.effective_threshold()
    }

    /// Appliquer la dissipation de l'activation.
    pub fn decay_activation(&mut self, rate: f64) {
        self.activation *= 1.0 - rate;
        self.activation = self.activation.max(0.0);
    }

    /// Mettre à jour la fatigue.
    pub fn update_fatigue(&mut self, gain: f64, recovery: f64) {
        if self.is_active() {
            self.fatigue += gain * self.activation;
        }
        self.fatigue *= 1.0 - recovery;
        self.fatigue = self.fatigue.clamp(0.0, 10.0);
    }

    /// Mettre à jour l'inhibition.
    pub fn update_inhibition(&mut self, gain: f64, decay: f64) {
        if self.is_active() {
            self.inhibition += gain;
        }
        self.inhibition *= 1.0 - decay;
        self.inhibition = self.inhibition.clamp(0.0, 10.0);
    }

    /// Mettre à jour la trace mémoire locale.
    pub fn update_trace(&mut self, trace_gain: f64, trace_decay: f64) {
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
    pub fn inject_stimulus(&mut self, intensity: f64) {
        self.activation += intensity;
    }
}
