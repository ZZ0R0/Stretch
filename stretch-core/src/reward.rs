use serde::{Deserialize, Serialize};

/// V4 : Système de récompense externe.
///
/// Gère le schedule de rewards et le reward courant.
/// Mode "supervised" : le reward est calculé automatiquement en comparant
/// la sortie du réseau à la cible attendue.
#[derive(Debug, Clone)]
pub struct RewardSystem {
    /// Reward courant r(t) ∈ [-1, +1]
    pub current: f32,
    /// Reward cumulé depuis le début
    pub cumulative: f32,
    /// Nombre total de récompenses attribuées
    pub count: usize,
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
}

fn default_reward_positive() -> f64 { 1.0 }
fn default_reward_negative() -> f64 { -1.0 }

impl Default for RewardConfig {
    fn default() -> Self {
        RewardConfig {
            reward_positive: default_reward_positive(),
            reward_negative: default_reward_negative(),
        }
    }
}

impl RewardSystem {
    pub fn new() -> Self {
        RewardSystem {
            current: 0.0,
            cumulative: 0.0,
            count: 0,
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
}
