use crate::config::StimulusConfig;
use crate::domain::Domain;

/// Injecter les stimuli prévus pour le tick courant.
pub fn inject_stimuli(domain: &mut Domain, stimuli: &[StimulusConfig], tick: usize) {
    for stim in stimuli {
        if tick < stim.start_tick || tick >= stim.end_tick {
            continue;
        }
        if stim.repeat_interval > 0 {
            let ticks_since_start = tick - stim.start_tick;
            if ticks_since_start % stim.repeat_interval != 0 {
                continue;
            }
        }
        if stim.node < domain.nodes.len() {
            domain.nodes[stim.node].inject_stimulus(stim.intensity);
        }
    }
}
