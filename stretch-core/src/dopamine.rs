use serde::{Deserialize, Serialize};

/// V4 : Système dopaminergique minimal.
///
/// Maintient un niveau tonique + phasique.
/// Le phasique est piloté par le reward externe.
/// Le signal total module la plasticité et le gating de consolidation.
#[derive(Debug, Clone)]
pub struct DopamineSystem {
    /// Niveau tonique de base (constant sauf modulation manuelle)
    pub tonic: f32,
    /// Composante phasique (burst/dip, décroît exponentiellement)
    pub phasic: f32,
    /// Niveau dopaminergique total = tonic + phasic
    pub level: f32,
}

/// Configuration dopaminergique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DopamineConfig {
    /// Niveau tonique de base
    #[serde(default = "default_da_tonic")]
    pub tonic: f64,
    /// Decay de la composante phasique par tick
    #[serde(default = "default_da_phasic_decay")]
    pub phasic_decay: f64,
    /// Gain : reward → dopamine phasique
    #[serde(default = "default_da_reward_gain")]
    pub reward_gain: f64,
    /// Gain de la dopamine sur la plasticité (eta_rew dans les formules)
    #[serde(default = "default_da_plasticity_gain")]
    pub plasticity_gain: f64,
    /// Seuil de dopamine pour le gating de consolidation
    #[serde(default = "default_da_consolidation_threshold")]
    pub consolidation_threshold: f64,
    /// Borne max de la composante phasique
    #[serde(default = "default_da_phasic_max")]
    pub phasic_max: f64,
    /// V4 : Décroissance spatiale de la dopamine — λ pour d_local = d_tonic + d_phasic × exp(-λ×dist)
    /// 0.0 = dopamine globale (pas de spatial), >0 = dopamine localisée au reward_center
    #[serde(default)]
    pub spatial_lambda: f64,
}

fn default_da_tonic() -> f64 { 0.1 }
fn default_da_phasic_decay() -> f64 { 0.15 }
fn default_da_reward_gain() -> f64 { 1.0 }
fn default_da_plasticity_gain() -> f64 { 0.01 }
fn default_da_consolidation_threshold() -> f64 { 0.3 }
fn default_da_phasic_max() -> f64 { 2.0 }

impl Default for DopamineConfig {
    fn default() -> Self {
        DopamineConfig {
            tonic: default_da_tonic(),
            phasic_decay: default_da_phasic_decay(),
            reward_gain: default_da_reward_gain(),
            plasticity_gain: default_da_plasticity_gain(),
            consolidation_threshold: default_da_consolidation_threshold(),
            phasic_max: default_da_phasic_max(),
            spatial_lambda: 0.0,
        }
    }
}

impl DopamineSystem {
    pub fn new(config: &DopamineConfig) -> Self {
        let tonic = config.tonic as f32;
        DopamineSystem {
            tonic,
            phasic: 0.0,
            level: tonic,
        }
    }

    /// Mettre à jour le système dopaminergique avec le reward courant.
    ///
    /// d_phasic(t+1) = (1 - decay) * d_phasic(t) + gain * r(t)
    /// d_total = d_tonic + d_phasic
    pub fn update(&mut self, reward: f64, config: &DopamineConfig) {
        self.phasic = (1.0 - config.phasic_decay as f32) * self.phasic
            + (config.reward_gain as f32) * (reward as f32);
        self.phasic = self.phasic.clamp(-(config.phasic_max as f32), config.phasic_max as f32);
        self.tonic = config.tonic as f32;
        self.level = self.tonic + self.phasic;
    }
}
