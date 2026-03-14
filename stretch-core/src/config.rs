use serde::{Deserialize, Serialize};

use crate::dopamine::DopamineConfig;
use crate::input::InputConfig;
use crate::output::OutputConfig;
use crate::reward::RewardConfig;

/// Configuration complète du système V3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub domain: DomainConfig,
    pub node_defaults: NodeDefaults,
    pub edge_defaults: EdgeDefaults,
    pub propagation: PropagationConfig,
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
    /// V3 : types neuronaux (E/I)
    #[serde(default)]
    pub neuron_types: NeuronTypesConfig,
    /// V3 : STDP
    #[serde(default)]
    pub stdp: StdpConfig,
    /// V3 : budget synaptique (normalisation)
    #[serde(default)]
    pub synaptic_budget: SynapticBudgetConfig,
    /// V4 : dopamine
    #[serde(default)]
    pub dopamine: DopamineConfig,
    /// V4 : récompense
    #[serde(default)]
    pub reward: RewardConfig,
    /// V4 : éligibilité
    #[serde(default)]
    pub eligibility: EligibilityConfig,
    /// V4 : interface d'entrée
    #[serde(default)]
    pub input: InputConfig,
    /// V4 : interface de sortie
    #[serde(default)]
    pub output: OutputConfig,
    /// V4.2 : compute backend configuration
    #[serde(default)]
    pub compute: ComputeConfig,
}

/// V4.2 : Compute backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeConfig {
    /// "auto" (try GPU, fall back to CPU), "cpu", or "gpu"
    #[serde(default = "default_compute_backend")]
    pub backend: String,
    /// Workgroup size for GPU compute shaders
    #[serde(default = "default_gpu_workgroup_size")]
    pub gpu_workgroup_size: u32,
}

fn default_compute_backend() -> String { "auto".into() }
fn default_gpu_workgroup_size() -> u32 { 256 }

impl Default for ComputeConfig {
    fn default() -> Self {
        ComputeConfig {
            backend: default_compute_backend(),
            gpu_workgroup_size: default_gpu_workgroup_size(),
        }
    }
}

impl SimConfig {
    /// Helper to get the backend preference string.
    pub fn backend_pref(&self) -> &str {
        &self.compute.backend
    }
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
    /// V3 : facteur de gain pour les neurones inhibiteurs (>0, appliqué en négatif)
    #[serde(default = "default_gain_inhibitory")]
    pub gain_inhibitory: f64,
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
    /// V3 : mode PID — "direct" (V2) ou "indirect" (V3)
    #[serde(default = "default_pid_mode")]
    pub pid_mode: String,
    /// V3 : coefficient de modulation du seuil par le PID indirect
    #[serde(default = "default_k_theta")]
    pub k_theta: f64,
    /// V3 : coefficient de modulation du gain par le PID indirect
    #[serde(default = "default_k_gain")]
    pub k_gain: f64,
}

impl Default for ZoneConfig {
    fn default() -> Self {
        ZoneConfig {
            num_zones: 8,
            partition_method: "voronoi".into(),
            target_activity: 0.3,
            kp: 0.5,
            ki: 0.05,
            kd: 0.1,
            pid_output_max: 2.0,
            pid_integral_max: 5.0,
            pid_mode: "direct".into(),
            k_theta: 0.3,
            k_gain: 0.2,
        }
    }
}

/// V2 : Configuration de consolidation mémoire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
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

fn default_gain_inhibitory() -> f64 { 0.8 }
fn default_pid_mode() -> String { "direct".into() }
fn default_k_theta() -> f64 { 0.3 }
fn default_k_gain() -> f64 { 0.2 }
fn default_inhibitory_fraction() -> f64 { 0.2 }
fn default_stdp_a_plus() -> f64 { 0.005 }
fn default_stdp_a_minus() -> f64 { 0.005 }
fn default_stdp_tau_plus() -> f64 { 20.0 }
fn default_stdp_tau_minus() -> f64 { 20.0 }
fn default_synaptic_budget() -> f64 { 30.0 }
fn default_eligibility_decay() -> f64 { 0.95 }
fn default_eligibility_max() -> f64 { 5.0 }

/// V4 : Traces d'éligibilité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityConfig {
    /// Taux de décroissance (gamma_e) : e_ij *= decay chaque tick
    #[serde(default = "default_eligibility_decay")]
    pub decay: f64,
    /// Plafond de la trace d'éligibilité
    #[serde(default = "default_eligibility_max")]
    pub max: f64,
}

impl Default for EligibilityConfig {
    fn default() -> Self {
        EligibilityConfig {
            decay: default_eligibility_decay(),
            max: default_eligibility_max(),
        }
    }
}

/// V3 : Configuration des types neuronaux E/I
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronTypesConfig {
    #[serde(default = "default_inhibitory_fraction")]
    pub inhibitory_fraction: f64,
}

impl Default for NeuronTypesConfig {
    fn default() -> Self {
        NeuronTypesConfig {
            inhibitory_fraction: 0.2,
        }
    }
}

/// V3 : Configuration STDP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdpConfig {
    #[serde(default = "default_stdp_a_plus")]
    pub a_plus: f64,
    #[serde(default = "default_stdp_a_minus")]
    pub a_minus: f64,
    #[serde(default = "default_stdp_tau_plus")]
    pub tau_plus: f64,
    #[serde(default = "default_stdp_tau_minus")]
    pub tau_minus: f64,
}

impl Default for StdpConfig {
    fn default() -> Self {
        StdpConfig {
            a_plus: 0.005,
            a_minus: 0.005,
            tau_plus: 20.0,
            tau_minus: 20.0,
        }
    }
}

/// V3 : Budget synaptique (normalisation des conductances sortantes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapticBudgetConfig {
    #[serde(default = "default_synaptic_budget")]
    pub budget: f64,
    /// Intervalle (en ticks) entre les renormalisations du budget.
    /// Plus grand = la plasticité reward a plus de temps pour s'exprimer.
    #[serde(default = "default_budget_interval")]
    pub interval: usize,
}

fn default_budget_interval() -> usize { 50 }

impl Default for SynapticBudgetConfig {
    fn default() -> Self {
        SynapticBudgetConfig {
            budget: 30.0,
            interval: default_budget_interval(),
        }
    }
}

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
                gain_inhibitory: 0.8,
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
            neuron_types: NeuronTypesConfig::default(),
            stdp: StdpConfig::default(),
            synaptic_budget: SynapticBudgetConfig::default(),
            dopamine: DopamineConfig::default(),
            reward: RewardConfig::default(),
            eligibility: EligibilityConfig::default(),
            input: InputConfig::default(),
            output: OutputConfig::default(),
            compute: ComputeConfig::default(),
        }
    }
}
