use serde::{Deserialize, Serialize};

/// Configuration complète du système V2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub domain: DomainConfig,
    pub node_defaults: NodeDefaults,
    pub edge_defaults: EdgeDefaults,
    pub propagation: PropagationConfig,
    pub plasticity: PlasticityConfig,
    pub dissipation: DissipationConfig,
    pub simulation: SimulationParams,
    #[serde(default)]
    pub stimuli: Vec<StimulusConfig>,
    /// V2 : configuration des zones et neurones de contrôle
    #[serde(default)]
    pub zones: ZoneConfig,
    /// V2 : configuration de la consolidation mémoire
    #[serde(default)]
    pub consolidation: ConsolidationConfig,
    /// V2 : liste des pacemakers
    #[serde(default)]
    pub pacemakers: Vec<PacemakerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Type de topologie : "grid2d", "random_sparse", "knn_3d", "radius_3d"
    pub topology: String,
    /// Nombre de nœuds (côté pour grid2d, total pour les autres)
    pub size: usize,
    /// Nombre moyen de voisins (pour random_sparse)
    #[serde(default = "default_avg_neighbors")]
    pub avg_neighbors: usize,
    /// Nombre de plus proches voisins (pour knn_3d)
    #[serde(default = "default_k_neighbors")]
    pub k_neighbors: usize,
    /// Rayon de connexion (pour radius_3d)
    #[serde(default = "default_radius")]
    pub radius: f64,
    /// Taille du domaine spatial 3D (cube de côté domain_extent)
    #[serde(default = "default_domain_extent")]
    pub domain_extent: f64,
    /// Graine aléatoire pour la génération du graphe
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefaults {
    pub activation: f64,
    pub threshold: f64,
    pub fatigue: f64,
    pub memory_trace: f64,
    pub excitability: f64,
    pub inhibition: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefaults {
    pub conductance: f64,
    pub plasticity: f64,
    pub decay: f64,
    pub conductance_min: f64,
    pub conductance_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationConfig {
    /// Type de noyau : "exponential", "gaussian"
    pub kernel: String,
    /// Facteur d'atténuation spatiale
    pub spatial_decay: f64,
    /// Gain de propagation global
    pub gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticityConfig {
    /// Taux de renforcement (Hebbian-like)
    pub reinforcement_rate: f64,
    /// Taux d'affaiblissement
    pub weakening_rate: f64,
    /// Seuil de co-activation pour renforcement
    pub coactivation_threshold: f64,
    /// Vitesse de décroissance des traces de co-activation
    pub coactivity_decay: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissipationConfig {
    /// Taux de décroissance de l'activation par tick
    pub activation_decay: f64,
    /// Gain de fatigue par activation
    pub fatigue_gain: f64,
    /// Taux de récupération de la fatigue
    pub fatigue_recovery: f64,
    /// Gain d'inhibition
    pub inhibition_gain: f64,
    /// Taux de décroissance de l'inhibition
    pub inhibition_decay: f64,
    /// Vitesse de décroissance des traces mémoire
    pub trace_decay: f64,
    /// Gain de trace mémoire par activation
    pub trace_gain: f64,
    /// Activation minimale (potentiel de repos) — empêche le flatline à 0
    #[serde(default = "default_activation_min")]
    pub activation_min: f64,
    /// Jitter aléatoire sur le taux de decay (fraction, ex: 0.15 = ±15%)
    #[serde(default)]
    pub decay_jitter: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Nombre total de ticks
    pub total_ticks: usize,
    /// Intervalle d'export des métriques (en ticks)
    pub snapshot_interval: usize,
    /// Graine globale pour la reproductibilité
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StimulusConfig {
    /// Nœud cible (index)
    pub node: usize,
    /// Tick de début
    pub start_tick: usize,
    /// Tick de fin (exclusif)
    pub end_tick: usize,
    /// Intensité du stimulus
    pub intensity: f64,
    /// Intervalle de répétition (0 = une seule fois par tick dans la plage)
    #[serde(default)]
    pub repeat_interval: usize,
}

/// V2 : Configuration des zones de contrôle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConfig {
    /// Activer le système de zones (false = mode V1 rétro-compatible)
    #[serde(default)]
    pub enabled: bool,
    /// Nombre de zones (neurones de contrôle)
    #[serde(default = "default_num_zones")]
    pub num_zones: usize,
    /// Méthode de partitionnement : "voronoi", "kmeans"
    #[serde(default = "default_partition_method")]
    pub partition_method: String,
    /// Consigne d'activité cible pour les NC
    #[serde(default = "default_target_activity")]
    pub target_activity: f64,
    /// Gain proportionnel du PID
    #[serde(default = "default_kp")]
    pub kp: f64,
    /// Gain intégral du PID
    #[serde(default = "default_ki")]
    pub ki: f64,
    /// Gain dérivé du PID
    #[serde(default = "default_kd")]
    pub kd: f64,
    /// Borne maximale de la sortie PID (anti-windup)
    #[serde(default = "default_pid_output_max")]
    pub pid_output_max: f64,
    /// Borne intégrale (anti-windup)
    #[serde(default = "default_pid_integral_max")]
    pub pid_integral_max: f64,
}

impl Default for ZoneConfig {
    fn default() -> Self {
        ZoneConfig {
            enabled: false,
            num_zones: 8,
            partition_method: "voronoi".into(),
            target_activity: 0.3,
            kp: 0.5,
            ki: 0.05,
            kd: 0.1,
            pid_output_max: 2.0,
            pid_integral_max: 5.0,
        }
    }
}

/// V2 : Configuration de consolidation mémoire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Activer la consolidation
    #[serde(default)]
    pub enabled: bool,
    /// Seuil de conductance pour démarrer la consolidation
    #[serde(default = "default_consolidation_threshold")]
    pub threshold: f64,
    /// Nombre de ticks au-dessus du seuil pour consolider
    #[serde(default = "default_consolidation_ticks")]
    pub ticks_required: usize,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        ConsolidationConfig {
            enabled: false,
            threshold: 2.5,
            ticks_required: 50,
        }
    }
}

/// V2 : Configuration d'un nœud pacemaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacemakerConfig {
    /// Nœud cible (index)
    pub node: usize,
    /// Amplitude de l'oscillation
    #[serde(default = "default_pacemaker_amplitude")]
    pub amplitude: f64,
    /// Fréquence (cycles par tick)
    #[serde(default = "default_pacemaker_frequency")]
    pub frequency: f64,
    /// Phase initiale (radians)
    #[serde(default)]
    pub phase: f64,
    /// Offset DC (activation de base)
    #[serde(default = "default_pacemaker_offset")]
    pub offset: f64,
}

fn default_num_zones() -> usize { 8 }
fn default_partition_method() -> String { "voronoi".into() }
fn default_target_activity() -> f64 { 0.3 }
fn default_kp() -> f64 { 0.5 }
fn default_ki() -> f64 { 0.05 }
fn default_kd() -> f64 { 0.1 }
fn default_pid_output_max() -> f64 { 2.0 }
fn default_pid_integral_max() -> f64 { 5.0 }
fn default_consolidation_threshold() -> f64 { 2.5 }
fn default_consolidation_ticks() -> usize { 50 }
fn default_pacemaker_amplitude() -> f64 { 0.3 }
fn default_pacemaker_frequency() -> f64 { 0.02 }
fn default_pacemaker_offset() -> f64 { 0.5 }

fn default_avg_neighbors() -> usize {
    6
}

fn default_k_neighbors() -> usize {
    12
}

fn default_radius() -> f64 {
    10.0
}

fn default_domain_extent() -> f64 {
    100.0
}

fn default_activation_min() -> f64 {
    0.01
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig {
            domain: DomainConfig {
                topology: "grid2d".into(),
                size: 20,
                avg_neighbors: 6,
                k_neighbors: 12,
                radius: 10.0,
                domain_extent: 100.0,
                seed: 42,
            },
            node_defaults: NodeDefaults {
                activation: 0.0,
                threshold: 0.4,
                fatigue: 0.0,
                memory_trace: 0.0,
                excitability: 1.0,
                inhibition: 0.0,
            },
            edge_defaults: EdgeDefaults {
                conductance: 1.0,
                plasticity: 1.0,
                decay: 0.01,
                conductance_min: 0.1,
                conductance_max: 5.0,
            },
            propagation: PropagationConfig {
                kernel: "exponential".into(),
                spatial_decay: 1.0,
                gain: 0.15,
            },
            plasticity: PlasticityConfig {
                reinforcement_rate: 0.01,
                weakening_rate: 0.002,
                coactivation_threshold: 0.2,
                coactivity_decay: 0.05,
            },
            dissipation: DissipationConfig {
                activation_decay: 0.35,
                fatigue_gain: 0.15,
                fatigue_recovery: 0.05,
                inhibition_gain: 0.08,
                inhibition_decay: 0.03,
                trace_decay: 0.005,
                trace_gain: 0.1,
                activation_min: 0.01,
                decay_jitter: 0.15,
            },
            simulation: SimulationParams {
                total_ticks: 500,
                snapshot_interval: 10,
                seed: 42,
            },
            stimuli: vec![StimulusConfig {
                node: 210, // centre d'une grille 20x20
                start_tick: 10,
                end_tick: 15,
                intensity: 1.0,
                repeat_interval: 0,
            }],
            zones: ZoneConfig::default(),
            consolidation: ConsolidationConfig::default(),
            pacemakers: Vec::new(),
        }
    }
}
