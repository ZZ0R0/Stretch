use serde::{Deserialize, Serialize};

use crate::config::EdgeDefaults;

/// État complet d'une liaison entre deux nœuds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    /// Facilité de propagation
    pub conductance: f32,
    /// Distance topologique / coût spatial
    pub distance: f32,
    /// Historique de co-activation
    pub coactivity_trace: f32,
    /// Compteur d'utilisation (instrumentation)
    pub usage_count: u64,
    /// V2 : ticks consécutifs au-dessus du seuil de consolidation
    pub consolidation_counter: usize,
    /// V2 : l'arête est-elle consolidée (decay désactivé) ?
    pub consolidated: bool,
    /// V4 : trace d'éligibilité pour l'apprentissage par récompense
    pub eligibility: f32,
}

impl Edge {
    pub fn new(from: usize, to: usize, distance: f64, defaults: &EdgeDefaults) -> Self {
        Edge {
            from,
            to,
            conductance: defaults.conductance as f32,
            distance: distance as f32,
            coactivity_trace: 0.0,
            usage_count: 0,
            consolidation_counter: 0,
            consolidated: false,
            eligibility: 0.0,
        }
    }

    /// Create an edge with only endpoints set, other fields at default.
    /// Used for rebuilding edges from GPU data.
    pub fn default_with_endpoints(from: usize, to: usize) -> Self {
        Edge {
            from,
            to,
            conductance: 1.0,
            distance: 0.0,
            coactivity_trace: 0.0,
            usage_count: 0,
            consolidation_counter: 0,
            consolidated: false,
            eligibility: 0.0,
        }
    }

    /// Décroissance de la trace de co-activation.
    pub fn decay_coactivity(&mut self, rate: f32) {
        self.coactivity_trace *= 1.0 - rate;
    }

    /// Enregistrer une co-activation.
    pub fn record_coactivation(&mut self, source_activation: f32, target_activation: f32) {
        let signal = source_activation.min(1.0) * target_activation.min(1.0);
        self.coactivity_trace += signal;
        self.coactivity_trace = self.coactivity_trace.min(10.0);
        self.usage_count += 1;
    }

    /// Mettre à jour la conductance selon la plasticité (Hebbian-like).
    /// Si l'arête est consolidée, seul le renforcement est possible (pas d'affaiblissement).
    pub fn update_conductance(&mut self, reinforcement_rate: f32, weakening_rate: f32, coact_threshold: f32, plasticity: f32, cond_min: f32, cond_max: f32) {
        if self.coactivity_trace > coact_threshold {
            let delta = reinforcement_rate * plasticity * (self.coactivity_trace - coact_threshold);
            self.conductance += delta;
        } else if !self.consolidated {
            let delta = weakening_rate * plasticity;
            self.conductance -= delta;
        }
        self.conductance = self.conductance.clamp(cond_min, cond_max);
    }

    /// V2 : mettre à jour le compteur de consolidation.
    pub fn update_consolidation(&mut self, threshold: f32, ticks_required: usize) {
        if self.consolidated {
            return;
        }
        if self.conductance >= threshold {
            self.consolidation_counter += 1;
            if self.consolidation_counter >= ticks_required {
                self.consolidated = true;
            }
        } else {
            // Reset si la conductance retombe sous le seuil
            self.consolidation_counter = 0;
        }
    }

    /// Décroissance lente de la conductance vers la valeur de base.
    pub fn decay_conductance(&mut self, rate: f32, baseline: f32) {
        self.conductance += rate * (baseline - self.conductance);
    }
}
