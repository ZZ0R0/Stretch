use std::time::Instant;

/// Named timing phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Phase {
    Zones = 0,
    StimInput = 1,
    Propagation = 2,
    Dissipation = 3,
    Plasticity = 4,
    ReadoutReward = 5,
    Metrics = 6,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Phase::Zones => "zones",
            Phase::StimInput => "stim+input",
            Phase::Propagation => "PROPAG",
            Phase::Dissipation => "dissip",
            Phase::Plasticity => "PLAST+STDP",
            Phase::ReadoutReward => "readout+rew",
            Phase::Metrics => "metrics",
        }
    }

    pub const ALL: [Phase; 7] = [
        Phase::Zones,
        Phase::StimInput,
        Phase::Propagation,
        Phase::Dissipation,
        Phase::Plasticity,
        Phase::ReadoutReward,
        Phase::Metrics,
    ];
}

/// Running statistics for a single phase.
#[derive(Debug, Clone)]
struct PhaseStats {
    count: u64,
    sum_us: u64,
    min_us: u64,
    max_us: u64,
}

impl PhaseStats {
    fn new() -> Self {
        PhaseStats {
            count: 0,
            sum_us: 0,
            min_us: u64::MAX,
            max_us: 0,
        }
    }

    fn record_us(&mut self, us: u64) {
        self.count += 1;
        self.sum_us += us;
        self.min_us = self.min_us.min(us);
        self.max_us = self.max_us.max(us);
    }

    fn mean_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_us as f64 / self.count as f64
        }
    }
}

/// Performance monitor tracking per-phase timing with min/mean/max stats.
///
/// Usage:
///     monitor.begin_tick();
///     // ... phase work ...
///     monitor.end_phase(Phase::Propagation);
///     // ... more phases ...
///     monitor.end_tick(tick);
pub struct PerfMonitor {
    stats: [PhaseStats; 7],
    tick_start: Instant,
    phase_start: Instant,
    report_interval: usize,
    last_tick_us: u64,
    window_start_tick: usize,
}

impl PerfMonitor {
    pub fn new(report_interval: usize) -> Self {
        let now = Instant::now();
        PerfMonitor {
            stats: std::array::from_fn(|_| PhaseStats::new()),
            tick_start: now,
            phase_start: now,
            report_interval,
            last_tick_us: 0,
            window_start_tick: 0,
        }
    }

    /// Call at the start of each tick.
    pub fn begin_tick(&mut self) {
        let now = Instant::now();
        self.tick_start = now;
        self.phase_start = now;
    }

    /// Call at the end of each phase. Records duration since last phase/tick start.
    pub fn end_phase(&mut self, phase: Phase) {
        let now = Instant::now();
        let us = (now - self.phase_start).as_micros() as u64;
        self.stats[phase as usize].record_us(us);
        self.phase_start = now;
    }

    /// Call at the end of each tick. Prints report at interval.
    pub fn end_tick(&mut self, tick: usize) {
        self.last_tick_us = (Instant::now() - self.tick_start).as_micros() as u64;

        if self.report_interval > 0 && tick > 0 && tick % self.report_interval == 0 {
            self.report(tick);
            self.reset_window(tick);
        }
    }

    /// Last tick duration in ms.
    pub fn last_tick_ms(&self) -> f64 {
        self.last_tick_us as f64 / 1000.0
    }

    fn report(&self, tick: usize) {
        let _window = tick - self.window_start_tick;
        let mut parts = Vec::new();
        for phase in Phase::ALL {
            let s = &self.stats[phase as usize];
            if s.count > 0 {
                parts.push(format!(
                    "{}:{:.2}/{:.2}/{:.2}",
                    phase.label(),
                    s.min_us as f64 / 1000.0,
                    s.mean_us() / 1000.0,
                    s.max_us as f64 / 1000.0,
                ));
            }
        }
        let total_mean: f64 = self.stats.iter().map(|s| s.mean_us()).sum::<f64>() / 1000.0;
        eprintln!(
            "[PERF ticks {}-{}] (min/avg/max ms) {} | total_avg:{:.2}ms",
            self.window_start_tick, tick, parts.join("  "), total_mean
        );
    }

    fn reset_window(&mut self, tick: usize) {
        for s in &mut self.stats {
            *s = PhaseStats::new();
        }
        self.window_start_tick = tick;
    }
}
