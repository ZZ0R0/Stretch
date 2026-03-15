use serde::{Deserialize, Serialize};

/// V4 : Système de récompense externe.
///
/// Gère le schedule de rewards et le reward courant.
/// Mode "supervised" : le reward est calculé automatiquement en comparant
/// la sortie du réseau à la cible attendue.
/// V5.2 : ajoute RPE (Reward Prediction Error) avec baseline glissante.
#[derive(Debug, Clone)]
pub struct RewardSystem {
    /// Reward courant r(t) ∈ [-1, +1]
    pub current: f32,
    /// Reward cumulé depuis le début
    pub cumulative: f32,
    /// Nombre total de récompenses attribuées
    pub count: usize,
    /// V5.2 : baseline (moyenne glissante exponentielle du reward)
    pub baseline: f32,
    /// V5.2 : dernier RPE δ = r_eff - baseline
    pub rpe_delta: f32,
}

/// Configuration du reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardConfig {
    /// Reward positif attribué quand la sortie est correcte
    #[serde(default = "default_reward_positive")]
    pub reward_positive: f64,
    /// Reward négatif attribué quand la sortie est incorrecte
    #[serde(default = "default_reward_negative")]
    pub reward_negative: f64,
    /// V5.2 : activer le RPE (Reward Prediction Error)
    #[serde(default)]
    pub rpe_enabled: bool,
    /// V5.2 : taux d'apprentissage de la baseline RPE (α)
    #[serde(default = "default_rpe_alpha")]
    pub rpe_alpha: f64,
    /// V5.2 : boost d'homéostasie quand δ < 0 (oubli accéléré)
    #[serde(default = "default_rho_boost")]
    pub rho_boost: f64,
    /// V5.2 : activer la modulation par la marge
    #[serde(default)]
    pub margin_modulation: bool,
    /// V5.2 : coefficient de modulation par la marge (β_M)
    #[serde(default = "default_margin_beta")]
    pub margin_beta: f64,
}

fn default_reward_positive() -> f64 { 1.0 }
fn default_reward_negative() -> f64 { -1.0 }
fn default_rpe_alpha() -> f64 { 0.05 }
fn default_rho_boost() -> f64 { 0.01 }
fn default_margin_beta() -> f64 { 0.1 }

impl Default for RewardConfig {
    fn default() -> Self {
        RewardConfig {
            reward_positive: default_reward_positive(),
            reward_negative: default_reward_negative(),
            rpe_enabled: false,
            rpe_alpha: default_rpe_alpha(),
            rho_boost: default_rho_boost(),
            margin_modulation: false,
            margin_beta: default_margin_beta(),
        }
    }
}

impl RewardSystem {
    pub fn new() -> Self {
        RewardSystem {
            current: 0.0,
            cumulative: 0.0,
            count: 0,
            baseline: 0.0,
            rpe_delta: 0.0,
        }
    }

    /// Attribuer un reward pour ce tick.
    pub fn set_reward(&mut self, reward: f64) {
        self.current = (reward as f32).clamp(-1.0, 1.0);
        self.cumulative += self.current;
        self.count += 1;
    }

    /// Réinitialiser le reward courant (appelé en début de tick).
    pub fn clear(&mut self) {
        self.current = 0.0;
    }

    /// V5.2 : Compute RPE δ = r_eff − baseline, update baseline.
    /// Returns δ for use as dopamine signal.
    pub fn compute_rpe(&mut self, r_eff: f32, alpha: f32) -> f32 {
        let delta = r_eff - self.baseline;
        self.baseline = (1.0 - alpha) * self.baseline + alpha * r_eff;
        self.rpe_delta = delta;
        delta
    }
}
