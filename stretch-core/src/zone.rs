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
    pub activity_mean: f32,
    /// Consigne d'activité
    pub target_activity: f32,
    /// PID : erreur courante
    pub error: f32,
    /// PID : intégrale cumulée
    pub integral: f32,
    /// PID : erreur précédente (pour le terme dérivé)
    pub error_prev: f32,
    /// PID : dernière sortie calculée
    pub output: f32,
    /// V3 : modulation du seuil (PID indirect)
    pub theta_mod: f32,
    /// V3 : modulation du gain (PID indirect)
    pub gain_mod: f32,
    /// C6 : nombre de ticks consécutifs avec |error| < ε
    pub stable_ticks: usize,
    /// C6 : est-ce que la zone est en mode skip ?
    pub is_stable: bool,
}

impl Zone {
    pub fn new(control_node: usize, members: Vec<usize>, target: f64) -> Self {
        Zone {
            control_node,
            members,
            activity_mean: 0.0,
            target_activity: target as f32,
            error: 0.0,
            integral: 0.0,
            error_prev: 0.0,
            output: 0.0,
            theta_mod: 0.0,
            gain_mod: 0.0,
            stable_ticks: 0,
            is_stable: false,
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
    pub gain_mods: Vec<f32>,
}

impl ZoneManager {
    /// Construire les zones par partitionnement de Voronoï autour de centres choisis.
    pub fn from_config(config: &ZoneConfig, domain: &Domain) -> Self {
        if config.num_zones == 0 {
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
    /// C6: skip stable zones (only measure every N ticks to check for reactivation).
    pub fn measure(&mut self, domain: &Domain) {
        self.zones.par_iter_mut().for_each(|zone| {
            if zone.members.is_empty() {
                zone.activity_mean = 0.0;
                return;
            }
            // C6: stable zones are measured every 10 ticks for reactivation check
            if zone.is_stable && zone.stable_ticks % 10 != 0 {
                zone.stable_ticks += 1;
                return;
            }
            let sum: f32 = zone.members.iter().map(|&i| domain.nodes[i].activation).sum();
            zone.activity_mean = sum / zone.members.len() as f32;
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
            // C6: skip stable zones
            if zone.is_stable {
                Self::check_stability(zone);
                if zone.is_stable { continue; }
            }

            let (u, error) = Self::compute_pid(zone, config);
            zone.error_prev = error;
            zone.error = error;
            zone.output = u;
            Self::update_stability(zone, error);

            for &member in &zone.members {
                domain.nodes[member].activation += u;
                domain.nodes[member].activation = domain.nodes[member].activation.clamp(0.0, 10.0);
            }
        }
    }

    /// V3 : PID indirect — moduler le seuil (θ) et le gain de propagation.
    fn regulate_indirect(&mut self, domain: &mut Domain, config: &ZoneConfig) {
        for zone in &mut self.zones {
            // C6: skip stable zones
            if zone.is_stable {
                Self::check_stability(zone);
                if zone.is_stable { continue; }
            }

            let (u, error) = Self::compute_pid(zone, config);
            zone.error_prev = error;
            zone.error = error;
            zone.output = u;
            Self::update_stability(zone, error);

            zone.theta_mod = -(config.k_theta as f32) * u;
            zone.gain_mod = (config.k_gain as f32) * u;

            for &member in &zone.members {
                domain.nodes[member].threshold_mod = zone.theta_mod;
            }

            for &member in &zone.members {
                self.gain_mods[member] = zone.gain_mod;
            }
        }
    }

    /// C6: Update stability tracking after PID computation.
    fn update_stability(zone: &mut Zone, error: f32) {
        const STABLE_EPSILON: f32 = 0.01;
        const STABLE_TICKS_REQUIRED: usize = 50;

        if error.abs() < STABLE_EPSILON {
            zone.stable_ticks += 1;
            if zone.stable_ticks >= STABLE_TICKS_REQUIRED {
                zone.is_stable = true;
            }
        } else {
            zone.stable_ticks = 0;
            zone.is_stable = false;
        }
    }

    /// C6: Check if a stable zone should be reactivated (measured periodically).
    fn check_stability(zone: &mut Zone) {
        const REACTIVATION_EPSILON: f32 = 0.05;
        let error = zone.target_activity - zone.activity_mean;
        if error.abs() > REACTIVATION_EPSILON {
            zone.is_stable = false;
            zone.stable_ticks = 0;
        }
    }

    /// Calcul PID commun aux modes direct et indirect.
    /// Retourne (u, error).
    fn compute_pid(zone: &mut Zone, config: &ZoneConfig) -> (f32, f32) {
        let error = zone.target_activity - zone.activity_mean;
        zone.integral = (zone.integral + error).clamp(-(config.pid_integral_max as f32), config.pid_integral_max as f32);
        let derivative = error - zone.error_prev;
        let u = (config.kp as f32) * error + (config.ki as f32) * zone.integral + (config.kd as f32) * derivative;
        let u = u.clamp(-(config.pid_output_max as f32), config.pid_output_max as f32);
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
        let sum: f32 = self.zones.iter().map(|z| z.activity_mean).sum();
        sum as f64 / self.zones.len() as f64
    }

    /// Erreur PID moyenne (absolue).
    pub fn mean_pid_error(&self) -> f64 {
        if self.zones.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.zones.iter().map(|z| z.error.abs()).sum();
        sum as f64 / self.zones.len() as f64
    }

    /// Sortie PID moyenne.
    pub fn mean_pid_output(&self) -> f64 {
        if self.zones.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.zones.iter().map(|z| z.output).sum();
        sum as f64 / self.zones.len() as f64
    }
}

fn euclidean_3d_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}
