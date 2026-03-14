use crate::config::PacemakerConfig;
use crate::domain::Domain;

/// V2 : Appliquer les oscillations des nœuds pacemaker.
/// Appelé à chaque tick, avant la propagation.
/// Equation : a_i(t) += A * sin(2π * f * t + φ) + offset
pub fn apply_pacemakers(domain: &mut Domain, pacemakers: &[PacemakerConfig], tick: usize) {
    let t = tick as f64;
    for pm in pacemakers {
        if pm.node >= domain.nodes.len() {
            continue;
        }
        let oscillation = pm.amplitude * (2.0 * std::f64::consts::PI * pm.frequency * t + pm.phase).sin();
        let injection = (pm.offset + oscillation) as f32;
        // Le pacemaker injecte une activation, bornée positivement
        if injection > 0.0 {
            domain.nodes[pm.node].activation += injection;
            domain.nodes[pm.node].activation = domain.nodes[pm.node].activation.min(10.0);
        }
        // Si injection < 0 : on peut inhiber légèrement
        else {
            domain.nodes[pm.node].activation = (domain.nodes[pm.node].activation + injection).max(0.0);
        }
    }
}
