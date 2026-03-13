use rayon::prelude::*;

use crate::config::ZoneConfig;
use crate::domain::Domain;

/// V3 : Zone de contrôle — un groupe de nœuds régulé par un contrôleur PID.
/// Supporte le mode "direct" (V2 : injection d'activation) et "indirect" (V3 : modulation θ/gain).
#[derive(Debug, Clone)]
pub struct Zone {
    /// Index du neurone de contrôle (position dans `positions`)
    pub control_node: usize,
    /// Indices des nœuds standards appartenant à cette zone
    pub members: Vec<usize>,
    /// Activité moyenne mesurée de la zone
    pub activity_mean: f64,
    /// Consigne d'activité
    pub target_activity: f64,
    /// PID : erreur courante
    pub error: f64,
    /// PID : intégrale cumulée
    pub integral: f64,
    /// PID : erreur précédente (pour le terme dérivé)
    pub error_prev: f64,
    /// PID : dernière sortie calculée
    pub output: f64,
    /// V3 : modulation du seuil (PID indirect)
    pub theta_mod: f64,
    /// V3 : modulation du gain (PID indirect)
    pub gain_mod: f64,
}

impl Zone {
    pub fn new(control_node: usize, members: Vec<usize>, target: f64) -> Self {
        Zone {
            control_node,
            members,
            activity_mean: 0.0,
            target_activity: target,
            error: 0.0,
            integral: 0.0,
            error_prev: 0.0,
            output: 0.0,
            theta_mod: 0.0,
            gain_mod: 0.0,
        }
    }
}

/// Gestionnaire de zones V3.
#[derive(Debug, Clone)]
pub struct ZoneManager {
    pub zones: Vec<Zone>,
    /// V3 : assignation nœud → zone (index dans `zones`)
    pub assignments: Vec<usize>,
    /// V3 : modulation de gain par nœud (utilisé par la propagation)
    pub gain_mods: Vec<f64>,
}

impl ZoneManager {
    /// Construire les zones par partitionnement de Voronoï autour de centres choisis.
    pub fn from_config(config: &ZoneConfig, domain: &Domain) -> Self {
        if !config.enabled || config.num_zones == 0 {
            let n = domain.num_nodes();
            return ZoneManager {
                zones: Vec::new(),
                assignments: vec![0; n],
                gain_mods: vec![0.0; n],
            };
        }

        let n = domain.num_nodes();
        let k = config.num_zones.min(n);

        // Choisir K centres répartis uniformément parmi les nœuds
        // (échantillonnage régulier pour reproductibilité sans RNG supplémentaire)
        let step = n / k;
        let centers: Vec<usize> = (0..k).map(|i| i * step).collect();

        // Assigner chaque nœud au centre le plus proche (Voronoï)
        let mut assignments = vec![0_usize; n];
        for i in 0..n {
            let pos_i = &domain.positions[i];
            let mut best_dist = f64::MAX;
            let mut best_zone = 0;
            for (z, &center) in centers.iter().enumerate() {
                let pos_c = &domain.positions[center];
                let dist = euclidean_3d_sq(pos_i, pos_c);
                if dist < best_dist {
                    best_dist = dist;
                    best_zone = z;
                }
            }
            assignments[i] = best_zone;
        }

        // Construire les zones
        let mut zone_members: Vec<Vec<usize>> = vec![Vec::new(); k];
        for (i, &z) in assignments.iter().enumerate() {
            // Le centre ne fait pas partie de ses propres membres (il est le contrôleur)
            if i != centers[z] {
                zone_members[z].push(i);
            }
        }

        let zones = centers
            .iter()
            .enumerate()
            .map(|(z, &center)| {
                Zone::new(center, zone_members[z].clone(), config.target_activity)
            })
            .collect();

        ZoneManager {
            zones,
            assignments,
            gain_mods: vec![0.0; n],
        }
    }

    /// Phase 0 du tick V2 : mesurer l'activité moyenne de chaque zone (parallèle).
    pub fn measure(&mut self, domain: &Domain) {
        self.zones.par_iter_mut().for_each(|zone| {
            if zone.members.is_empty() {
                zone.activity_mean = 0.0;
                return;
            }
            let sum: f64 = zone.members.iter().map(|&i| domain.nodes[i].activation).sum();
            zone.activity_mean = sum / zone.members.len() as f64;
        });
    }

    /// Phase 1 du tick : calculer la correction PID.
    /// Dispatche vers le mode direct (V2) ou indirect (V3) selon config.pid_mode.
    pub fn regulate(&mut self, domain: &mut Domain, config: &ZoneConfig) {
        if config.pid_mode == "indirect" {
            self.regulate_indirect(domain, config);
        } else {
            self.regulate_direct(domain, config);
        }
    }

    /// V2 : PID direct — injecter la correction dans l'activation des nœuds.
    fn regulate_direct(&mut self, domain: &mut Domain, config: &ZoneConfig) {
        for zone in &mut self.zones {
            let (u, error) = Self::compute_pid(zone, config);
            zone.error_prev = error;
            zone.error = error;
            zone.output = u;

            for &member in &zone.members {
                domain.nodes[member].activation += u;
                domain.nodes[member].activation = domain.nodes[member].activation.clamp(0.0, 10.0);
            }
        }
    }

    /// V3 : PID indirect — moduler le seuil (θ) et le gain de propagation.
    /// Au lieu d'injecter de l'activation, on ajuste :
    ///   θ_mod = -k_theta × u  (abaisser le seuil si activité trop basse)
    ///   g_mod = +k_gain × u   (augmenter le gain si activité trop basse)
    fn regulate_indirect(&mut self, domain: &mut Domain, config: &ZoneConfig) {
        for zone in &mut self.zones {
            let (u, error) = Self::compute_pid(zone, config);
            zone.error_prev = error;
            zone.error = error;
            zone.output = u;

            // V3 : modulations indirectes
            zone.theta_mod = -config.k_theta * u;
            zone.gain_mod = config.k_gain * u;

            // Appliquer θ_mod aux nœuds de la zone
            for &member in &zone.members {
                domain.nodes[member].threshold_mod = zone.theta_mod;
            }

            // Stocker gain_mod par nœud pour la propagation
            for &member in &zone.members {
                self.gain_mods[member] = zone.gain_mod;
            }
        }
    }

    /// Calcul PID commun aux modes direct et indirect.
    /// Retourne (u, error).
    fn compute_pid(zone: &mut Zone, config: &ZoneConfig) -> (f64, f64) {
        let error = zone.target_activity - zone.activity_mean;
        zone.integral = (zone.integral + error).clamp(-config.pid_integral_max, config.pid_integral_max);
        let derivative = error - zone.error_prev;
        let u = config.kp * error + config.ki * zone.integral + config.kd * derivative;
        let u = u.clamp(-config.pid_output_max, config.pid_output_max);
        (u, error)
    }

    /// Nombre de zones actives.
    pub fn num_zones(&self) -> usize {
        self.zones.len()
    }

    /// Activité moyenne globale de toutes les zones.
    pub fn global_activity_mean(&self) -> f64 {
        if self.zones.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.zones.iter().map(|z| z.activity_mean).sum();
        sum / self.zones.len() as f64
    }

    /// Erreur PID moyenne (absolue).
    pub fn mean_pid_error(&self) -> f64 {
        if self.zones.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.zones.iter().map(|z| z.error.abs()).sum();
        sum / self.zones.len() as f64
    }

    /// Sortie PID moyenne.
    pub fn mean_pid_output(&self) -> f64 {
        if self.zones.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.zones.iter().map(|z| z.output).sum();
        sum / self.zones.len() as f64
    }
}

fn euclidean_3d_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}
