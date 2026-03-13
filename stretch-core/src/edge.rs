use serde::{Deserialize, Serialize};

use crate::config::EdgeDefaults;

/// État complet d'une liaison entre deux nœuds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    /// Facilité de propagation
    pub conductance: f64,
    /// Distance topologique / coût spatial
    pub distance: f64,
    /// Historique de co-activation
    pub coactivity_trace: f64,
    /// Capacité de la liaison à évoluer
    pub plasticity: f64,
    /// Vitesse d'oubli / retour à la ligne de base
    pub decay: f64,
    /// Compteur d'utilisation (instrumentation)
    pub usage_count: u64,
    /// Bornes de conductance
    pub conductance_min: f64,
    pub conductance_max: f64,
}

impl Edge {
    pub fn new(from: usize, to: usize, distance: f64, defaults: &EdgeDefaults) -> Self {
        Edge {
            from,
            to,
            conductance: defaults.conductance,
            distance,
            coactivity_trace: 0.0,
            plasticity: defaults.plasticity,
            decay: defaults.decay,
            usage_count: 0,
            conductance_min: defaults.conductance_min,
            conductance_max: defaults.conductance_max,
        }
    }

    /// Décroissance de la trace de co-activation.
    pub fn decay_coactivity(&mut self, rate: f64) {
        self.coactivity_trace *= 1.0 - rate;
    }

    /// Enregistrer une co-activation.
    pub fn record_coactivation(&mut self, source_activation: f64, target_activation: f64) {
        let signal = source_activation.min(1.0) * target_activation.min(1.0);
        self.coactivity_trace += signal;
        self.coactivity_trace = self.coactivity_trace.min(10.0);
        self.usage_count += 1;
    }

    /// Mettre à jour la conductance selon la plasticité (Hebbian-like).
    pub fn update_conductance(&mut self, reinforcement_rate: f64, weakening_rate: f64, coact_threshold: f64) {
        if self.coactivity_trace > coact_threshold {
            // Renforcement
            let delta = reinforcement_rate * self.plasticity * (self.coactivity_trace - coact_threshold);
            self.conductance += delta;
        } else {
            // Affaiblissement lent
            let delta = weakening_rate * self.plasticity;
            self.conductance -= delta;
        }
        self.conductance = self.conductance.clamp(self.conductance_min, self.conductance_max);
    }

    /// Décroissance lente de la conductance vers la valeur de base.
    pub fn decay_conductance(&mut self, rate: f64, baseline: f64) {
        self.conductance += rate * (baseline - self.conductance);
    }
}
