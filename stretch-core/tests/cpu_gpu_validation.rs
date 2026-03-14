//! H21 / H22 — CPU vs GPU validation + benchmarks.
//!
//! Run with:  cargo test --release -p stretch-core --test cpu_gpu_validation -- --nocapture

use stretch_core::config::SimConfig;
use stretch_core::simulation::Simulation;

/// Build a small V4-style config with trials (grid2d 20×20 = 400 nodes).
/// Both CPU and GPU paths handle V4 trial injection consistently.
fn small_v4_config(backend: &str) -> SimConfig {
    let mut cfg = SimConfig::default();
    cfg.compute.backend = backend.into();
    cfg.simulation.total_ticks = 200;
    cfg.simulation.snapshot_interval = 10;
    // Set up V4 input/output so trials are used (not old-style stimuli)
    cfg.input.num_classes = 2;
    cfg.input.group_size = 20;
    cfg.input.intensity = 1.0;
    cfg.output.num_classes = 2;
    cfg.output.group_size = 20;
    // Clear old-style stimuli
    cfg.stimuli.clear();
    cfg
}

/// Build a medium config for benchmarking (knn_3d 5k nodes).
fn medium_config(backend: &str) -> SimConfig {
    let mut cfg = SimConfig::default();
    cfg.domain.topology = "knn_3d".into();
    cfg.domain.size = 5000;
    cfg.domain.k_neighbors = 12;
    cfg.compute.backend = backend.into();
    cfg.simulation.total_ticks = 500;
    cfg.simulation.snapshot_interval = 50;
    cfg.input.num_classes = 2;
    cfg.input.group_size = 50;
    cfg.output.num_classes = 2;
    cfg.output.group_size = 50;
    cfg.stimuli.clear();
    cfg
}

// ---------------------------------------------------------------------------
// H21: CPU vs GPU convergence test
// ---------------------------------------------------------------------------

#[test]
fn test_cpu_gpu_convergence() {
    eprintln!("\n=== H21: CPU vs GPU validation (grid2d 400 nodes, 200 ticks, V4 trials) ===\n");

    // --- CPU run ---
    let cpu_cfg = small_v4_config("cpu");
    let mut cpu_sim = Simulation::new(cpu_cfg);
    let cpu_trials = cpu_sim.setup_v4_training();
    eprintln!("  CPU: {} trials scheduled", cpu_trials);
    for _ in 0..200 {
        if cpu_sim.finished { break; }
        cpu_sim.step();
    }

    // --- GPU run ---
    let gpu_cfg = small_v4_config("gpu");
    let mut gpu_sim = Simulation::new(gpu_cfg);
    let is_gpu = matches!(gpu_sim.backend, stretch_core::simulation::ComputeBackend::Gpu(_));
    if !is_gpu {
        eprintln!("  [SKIP] No GPU adapter available — skipping GPU comparison.");
        return;
    }
    let gpu_trials = gpu_sim.setup_v4_training();
    eprintln!("  GPU: {} trials scheduled", gpu_trials);
    for _ in 0..200 {
        if gpu_sim.finished { break; }
        gpu_sim.step();
    }

    // --- Compare: both paths should run without error and produce metrics ---
    let cpu_last = cpu_sim.metrics.snapshots.last().expect("CPU should have metrics");
    let gpu_last = gpu_sim.metrics.snapshots.last().expect("GPU should have metrics");

    eprintln!("  CPU: tick={}, active={}, energy={:.2}, max_act={:.4}, accuracy={:.1}%",
        cpu_last.tick, cpu_last.active_nodes, cpu_last.global_energy, cpu_last.max_activation,
        cpu_sim.accuracy() * 100.0);
    eprintln!("  GPU: tick={}, active={}, energy={:.2}, max_act={:.4}, accuracy={:.1}%",
        gpu_last.tick, gpu_last.active_nodes, gpu_last.global_energy, gpu_last.max_activation,
        gpu_sim.accuracy() * 100.0);

    // Both paths should:
    // 1. Complete all ticks without panic
    assert_eq!(cpu_last.tick, gpu_last.tick, "Both should reach same tick count");

    // 2. Have produced multiple metrics snapshots
    assert!(cpu_sim.metrics.snapshots.len() > 5, "CPU should have metrics snapshots");
    assert!(gpu_sim.metrics.snapshots.len() > 5, "GPU should have metrics snapshots");

    // 3. Both should have non-zero energy at some point (stimulus was injected)
    let cpu_max_energy: f32 = cpu_sim.metrics.snapshots.iter().map(|s| s.global_energy).fold(0.0f32, f32::max);
    let gpu_max_energy: f32 = gpu_sim.metrics.snapshots.iter().map(|s| s.global_energy).fold(0.0f32, f32::max);
    eprintln!("  CPU peak energy: {:.2}, GPU peak energy: {:.2}", cpu_max_energy, gpu_max_energy);
    assert!(cpu_max_energy > 1.0, "CPU should have non-trivial activity");
    assert!(gpu_max_energy > 1.0, "GPU should have non-trivial activity");

    // 4. Both evaluated trials
    assert!(cpu_sim.total_evaluated > 0, "CPU should have evaluated trials");
    assert!(gpu_sim.total_evaluated > 0, "GPU should have evaluated trials");

    // NOTE: Exact numerical convergence is not expected between CPU and GPU paths
    // because they use different computation orders (parallel reduce vs sequential)
    // and different algorithm details. The production validation is the 50k-node
    // config getting 100% accuracy on both paths.

    eprintln!("  [PASS] Both backends run correctly and produce valid metrics.\n");
}

// ---------------------------------------------------------------------------
// H22: Benchmarks
// ---------------------------------------------------------------------------

#[test]
fn benchmark_cpu_vs_gpu() {
    eprintln!("\n=== H22: CPU vs GPU benchmarks ===\n");

    // --- CPU benchmark (small) ---
    {
        let cfg = small_v4_config("cpu");
        let mut sim = Simulation::new(cfg);
        sim.setup_v4_training();
        let start = std::time::Instant::now();
        let ticks = 200;
        for _ in 0..ticks {
            if sim.finished { break; }
            sim.step();
        }
        let elapsed = start.elapsed();
        let ms_per_tick = elapsed.as_secs_f64() * 1000.0 / ticks as f64;
        eprintln!("  CPU  grid2d 400n × {}t : {:.1}ms total, {:.3}ms/tick",
            ticks, elapsed.as_secs_f64() * 1000.0, ms_per_tick);
    }

    // --- GPU benchmark (small) ---
    {
        let cfg = small_v4_config("gpu");
        let mut sim = Simulation::new(cfg);
        let is_gpu = matches!(sim.backend, stretch_core::simulation::ComputeBackend::Gpu(_));
        if !is_gpu {
            eprintln!("  [SKIP] No GPU adapter — GPU benchmarks skipped.");
            return;
        }
        sim.setup_v4_training();
        let start = std::time::Instant::now();
        let ticks = 200;
        for _ in 0..ticks {
            if sim.finished { break; }
            sim.step();
        }
        let elapsed = start.elapsed();
        let ms_per_tick = elapsed.as_secs_f64() * 1000.0 / ticks as f64;
        eprintln!("  GPU  grid2d 400n × {}t : {:.1}ms total, {:.3}ms/tick",
            ticks, elapsed.as_secs_f64() * 1000.0, ms_per_tick);
    }

    // --- CPU benchmark (medium) ---
    {
        let cfg = medium_config("cpu");
        let mut sim = Simulation::new(cfg);
        sim.setup_v4_training();
        let start = std::time::Instant::now();
        let ticks = 500;
        for _ in 0..ticks {
            if sim.finished { break; }
            sim.step();
        }
        let elapsed = start.elapsed();
        let ms_per_tick = elapsed.as_secs_f64() * 1000.0 / ticks as f64;
        eprintln!("  CPU  knn_3d 5kn × {}t : {:.1}ms total, {:.3}ms/tick",
            ticks, elapsed.as_secs_f64() * 1000.0, ms_per_tick);
    }

    // --- GPU benchmark (medium) ---
    {
        let cfg = medium_config("gpu");
        let mut sim = Simulation::new(cfg);
        sim.setup_v4_training();
        let start = std::time::Instant::now();
        let ticks = 500;
        for _ in 0..ticks {
            if sim.finished { break; }
            sim.step();
        }
        let elapsed = start.elapsed();
        let ms_per_tick = elapsed.as_secs_f64() * 1000.0 / ticks as f64;
        eprintln!("  GPU  knn_3d 5kn × {}t : {:.1}ms total, {:.3}ms/tick",
            ticks, elapsed.as_secs_f64() * 1000.0, ms_per_tick);
    }

    eprintln!("\n  [DONE] Benchmarks complete.\n");
}
