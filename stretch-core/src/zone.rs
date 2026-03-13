use crate::config::ZoneConfig;
use crate::domain::Domain;

/// V2 : Zone de contrôle — un groupe de nœuds régulé par un contrôleur PID.
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
        }
    }
}

/// Gestionnaire de zones V2.
#[derive(Debug, Clone)]
pub struct ZoneManager {
    pub zones: Vec<Zone>,
}

impl ZoneManager {
    /// Construire les zones par partitionnement de Voronoï autour de centres choisis.
    pub fn from_config(config: &ZoneConfig, domain: &Domain) -> Self {
        if !config.enabled || config.num_zones == 0 {
            return ZoneManager { zones: Vec::new() };
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

        ZoneManager { zones }
    }

    /// Phase 0 du tick V2 : mesurer l'activité moyenne de chaque zone.
    pub fn measure(&mut self, domain: &Domain) {
        for zone in &mut self.zones {
            if zone.members.is_empty() {
                zone.activity_mean = 0.0;
                continue;
            }
            let sum: f64 = zone.members.iter().map(|&i| domain.nodes[i].activation).sum();
            zone.activity_mean = sum / zone.members.len() as f64;
        }
    }

    /// Phase 1 du tick V2 : calculer la correction PID et l'injecter.
    pub fn regulate(&mut self, domain: &mut Domain, config: &ZoneConfig) {
        for zone in &mut self.zones {
            // Erreur = consigne - mesure
            let error = zone.target_activity - zone.activity_mean;

            // Terme intégral (avec anti-windup)
            zone.integral = (zone.integral + error).clamp(-config.pid_integral_max, config.pid_integral_max);

            // Terme dérivé
            let derivative = error - zone.error_prev;

            // Sortie PID
            let u = config.kp * error + config.ki * zone.integral + config.kd * derivative;
            let u = u.clamp(-config.pid_output_max, config.pid_output_max);

            // Mémoriser pour le prochain tick
            zone.error_prev = error;
            zone.error = error;
            zone.output = u;

            // Injecter la correction dans tous les nœuds de la zone
            // Chaque nœud reçoit u directement (correction par nœud)
            for &member in &zone.members {
                domain.nodes[member].activation += u;
                domain.nodes[member].activation = domain.nodes[member].activation.max(0.0);
                domain.nodes[member].activation = domain.nodes[member].activation.min(10.0);
            }
        }
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
