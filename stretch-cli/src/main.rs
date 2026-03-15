use std::env;
use std::fs;

use stretch_core::config::{SimConfig, V5TaskMode};
use stretch_core::diagnostics;
use stretch_core::metrics::MetricsLog;
use stretch_core::simulation::{self, Simulation, NullObserver};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse --seeds N argument (multi-seed mode)
    let seeds_count = parse_seeds_arg(&args);

    let config = if args.len() > 1 && args[1] != "--gen-config" && args[1] != "--seeds" {
        let config_path = &args[1];
        let content = fs::read_to_string(config_path)
            .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", config_path, e));
        toml::from_str::<SimConfig>(&content)
            .unwrap_or_else(|e| panic!("Erreur de parsing config : {}", e))
    } else if args.len() > 2 && args[2] != "--seeds" {
        // --seeds N config.toml  OR  config.toml --seeds N
        let config_path = if args[1] == "--seeds" { &args[3] } else { &args[1] };
        let content = fs::read_to_string(config_path)
            .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", config_path, e));
        toml::from_str::<SimConfig>(&content)
            .unwrap_or_else(|e| panic!("Erreur de parsing config : {}", e))
    } else {
        if args.iter().any(|a| a == "--gen-config") {
            let default_config = SimConfig::default();
            let toml_str =
                toml::to_string_pretty(&default_config).expect("Erreur de sérialisation TOML");
            let gen_path = "default_config.toml";
            fs::write(gen_path, &toml_str).expect("Impossible d'écrire la config");
            println!("Config par défaut générée : {}", gen_path);
            return;
        }
        println!("Aucun fichier de config fourni, utilisation des valeurs par défaut.");
        println!("Usage: stretch-cli <config.toml> [--seeds N]\n");
        SimConfig::default()
    };

    if let Some(n_seeds) = seeds_count {
        run_multi_seed(&config, n_seeds);
    } else {
        run_single(&config);
    }
}

/// Parse --seeds N from CLI arguments. Returns Some(N) if found.
fn parse_seeds_arg(args: &[String]) -> Option<usize> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--seeds" {
            if let Some(val) = args.get(i + 1) {
                return val.parse::<usize>().ok();
            }
        }
    }
    None
}

/// Run N simulations with different seeds, aggregate results to CSV.
fn run_multi_seed(base_config: &SimConfig, n_seeds: usize) {
    let base_seed = base_config.domain.seed;
    let is_v5 = base_config.v5_task.task_mode != V5TaskMode::Legacy;
    let version_tag = if is_v5 {
        format!("V5 {:?}", base_config.v5_task.task_mode)
    } else {
        "V4".to_string()
    };

    println!("[Multi-seed] Running {} seeds for {} mode", n_seeds, version_tag);

    let mut all_accuracies = Vec::with_capacity(n_seeds);

    for s in 0..n_seeds {
        let seed = base_seed + s as u64;
        let mut config = base_config.clone();
        config.domain.seed = seed;

        let mut sim = Simulation::new(config.clone());
        let n_trials = if is_v5 {
            sim.setup_v5_training()
        } else {
            sim.setup_v4_training()
        };

        println!("[Seed {}] {} trials, seed={}", s, n_trials, seed);

        let mut observer = NullObserver;
        let result = simulation::run_simulation_loop(sim, &mut observer);

        // Extract accuracy from last 20% of trials
        let accuracy = compute_final_accuracy(&result.metrics, 0.2);
        all_accuracies.push(accuracy);
        println!("[Seed {}] accuracy={:.1}%", s, accuracy * 100.0);
    }

    // Aggregate statistics
    let mean = all_accuracies.iter().sum::<f64>() / n_seeds as f64;
    let variance = all_accuracies.iter().map(|&a| (a - mean).powi(2)).sum::<f64>() / n_seeds as f64;
    let std_dev = variance.sqrt();

    println!("\n=== Multi-seed Results ({} seeds) ===", n_seeds);
    println!("  mean accuracy: {:.1}%", mean * 100.0);
    println!("  std dev:       {:.1}%", std_dev * 100.0);
    println!("  min:           {:.1}%", all_accuracies.iter().cloned().fold(f64::INFINITY, f64::min) * 100.0);
    println!("  max:           {:.1}%", all_accuracies.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 100.0);

    // Export CSV
    let csv_path = "multi_seed_results.csv";
    let mut csv = String::from("seed,accuracy\n");
    for (i, &acc) in all_accuracies.iter().enumerate() {
        csv.push_str(&format!("{},{:.6}\n", base_seed + i as u64, acc));
    }
    csv.push_str(&format!("mean,{:.6}\n", mean));
    csv.push_str(&format!("std,{:.6}\n", std_dev));
    fs::write(csv_path, &csv).expect("Failed to write CSV");
    println!("Results exported to {}", csv_path);
}

/// Compute accuracy over the last `frac` of snapshots from metrics log.
fn compute_final_accuracy(metrics: &MetricsLog, frac: f64) -> f64 {
    let n = metrics.snapshots.len();
    if n == 0 { return 0.0; }
    let start = (n as f64 * (1.0 - frac)) as usize;
    let mut correct = 0usize;
    let mut total = 0usize;
    for snap in &metrics.snapshots[start..] {
        if snap.output_decision.is_some() {
            total += 1;
            if snap.current_reward > 0.0 {
                correct += 1;
            }
        }
    }
    if total == 0 { 0.0 } else { correct as f64 / total as f64 }
}

fn run_single(config: &SimConfig) {
    // V4/V5 : lancement en mode entraînement avec trials
    let mut sim = Simulation::new(config.clone());

    let is_v5 = config.v5_task.task_mode != V5TaskMode::Legacy;
    let n_trials = if is_v5 {
        sim.setup_v5_training()
    } else {
        sim.setup_v4_training()
    };

    // Spatial diagnostics
    print_spatial_diagnostics(&sim);

    let trial_period = if n_trials > 1 {
        sim.trials[1].start_tick - sim.trials[0].start_tick
    } else { 0 };
    let version_tag = if is_v5 {
        format!("V5 {:?}", config.v5_task.task_mode)
    } else {
        "V4".to_string()
    };
    println!("[{} Training] {} trials programmés ({} classes, période={})",
        version_tag, n_trials, config.input.num_classes, trial_period);

    let mut observer = NullObserver;
    let result = simulation::run_simulation_loop(sim, &mut observer);

    // Export métriques JSON
    let json =
        serde_json::to_string_pretty(&result.metrics).expect("Erreur de sérialisation JSON");
    fs::write("metrics_output.json", &json).expect("Impossible d'écrire metrics_output.json");
    println!("\nMétriques exportées dans : metrics_output.json");

    // Export traces nœuds
    let trace_data: Vec<serde_json::Value> = result
        .domain
        .nodes
        .iter()
        .filter(|n| n.memory_trace > 0.001)
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "memory_trace": n.memory_trace,
                "activation_count": n.activation_count,
                "excitability": n.excitability,
                "fatigue": n.fatigue
            })
        })
        .collect();
    let traces_json =
        serde_json::to_string_pretty(&trace_data).expect("Erreur de sérialisation traces");
    fs::write("node_traces.json", &traces_json).expect("Impossible d'écrire node_traces.json");
    println!("Traces nœuds exportées dans : node_traces.json");

    // Conductance statistics
    let conds: Vec<f32> = result.domain.edges.iter().map(|e| e.conductance).collect();
    let n_edges = conds.len();
    let mean_c = conds.iter().sum::<f32>() / n_edges as f32;
    let min_c = conds.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_c = conds.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let above = conds.iter().filter(|&&c| c > 1.05).count();
    let below = conds.iter().filter(|&&c| c < 0.95).count();
    let elig: Vec<f32> = result.domain.edges.iter().map(|e| e.eligibility).collect();
    let max_e = elig.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let nonzero_e = elig.iter().filter(|&&e| e.abs() > 1e-6).count();
    println!("\n=== Conductance Stats ===");
    println!("  n_edges: {}, mean: {:.6}, min: {:.6}, max: {:.6}", n_edges, mean_c, min_c, max_c);
    println!("  above 1.05: {}, below 0.95: {}", above, below);
    println!("  max_eligibility: {:.6}, nonzero_elig: {}", max_e, nonzero_e);

    // Export modified conductances only
    let edge_data: Vec<serde_json::Value> = result
        .domain
        .edges
        .iter()
        .filter(|e| (e.conductance - 1.0).abs() > 0.01 || e.eligibility.abs() > 0.001)
        .map(|e| {
            serde_json::json!({
                "from": e.from,
                "to": e.to,
                "conductance": e.conductance,
                "eligibility": e.eligibility,
                "consolidated": e.consolidated
            })
        })
        .collect();
    let edges_json =
        serde_json::to_string_pretty(&edge_data).expect("Erreur de sérialisation edges");
    fs::write("edge_conductances.json", &edges_json)
        .expect("Impossible d'écrire edge_conductances.json");
    println!("  modified edges exported: {}", edge_data.len());

    // === V5 Diagnostics ===
    if is_v5 && config.v5_diagnostics.path_tracer {
        eprintln!("[V5] Running post-simulation diagnostics...");

        let extent = config.domain.domain_extent;
        let group_size = config.input.group_size;

        let placement = stretch_core::task::place_io(
            &result.domain,
            &config.v5_task.task_mode,
            config.input.num_classes,
            group_size,
            extent,
        );

        let _initial_conds: Option<Vec<f32>> = None; // Not available post-sim
        let report = diagnostics::run_diagnostics(
            &result.domain,
            &placement.input_groups,
            &placement.output_groups,
            &placement.target_mapping,
            false, // No CT without initial conductances
            None,
        );
        diagnostics::print_diagnostic_report(&report);

        // Export diagnostic report
        let diag_json = serde_json::json!({
            "route_scores_target": report.route_scores_target,
            "route_scores_competitor": report.route_scores_competitor,
            "directional_indices": report.directional_indices,
            "topological_coherence": report.topological_coherence,
            "sustain_ratio": report.sustain_ratio,
        });
        fs::write("v5_diagnostics.json",
            serde_json::to_string_pretty(&diag_json).unwrap())
            .expect("Failed to write v5_diagnostics.json");
        println!("V5 diagnostics exported to v5_diagnostics.json");
    }

    // Histogramme traces
    let histogram = MetricsLog::trace_histogram(&result.domain, 10);
    println!("\nHistogramme des traces mémoire :");
    for (lower, count) in &histogram {
        let bar: String = std::iter::repeat('#').take(*count / 5 + 1).collect();
        println!("  [{:.3}, ...) : {:>5} {}", lower, count, bar);
    }
}

fn print_spatial_diagnostics(sim: &Simulation) {
    if let (Some(ref enc), Some(ref reader)) = (&sim.input_encoder, &sim.output_reader) {
        let d = &sim.domain;
        let groups: [(&Vec<usize>, &str); 4] = [
            (&enc.groups[0], "input-0"), (&enc.groups[1], "input-1"),
            (&reader.groups[0], "output-0"), (&reader.groups[1], "output-1"),
        ];
        println!("\n=== I/O Spatial Layout (spatial selection) ===");
        for (nodes, name) in &groups {
            let (mut sx, mut sy, mut sz) = (0.0_f64, 0.0_f64, 0.0_f64);
            for &i in nodes.iter() {
                let p = d.positions[i];
                sx += p[0]; sy += p[1]; sz += p[2];
            }
            let n = nodes.len() as f64;
            println!("  {} ({} nodes): centroid ({:.1}, {:.1}, {:.1})",
                name, nodes.len(), sx/n, sy/n, sz/n);
        }
        for (inodes, iname) in &groups[0..2] {
            for (onodes, oname) in &groups[2..4] {
                let mut total_d = 0.0_f64;
                let mut count = 0;
                for &i in inodes.iter() {
                    for &o in onodes.iter() {
                        let dist = ((d.positions[i][0]-d.positions[o][0]).powi(2)
                            + (d.positions[i][1]-d.positions[o][1]).powi(2)
                            + (d.positions[i][2]-d.positions[o][2]).powi(2)).sqrt();
                        total_d += dist;
                        count += 1;
                    }
                }
                println!("  dist {} → {}: {:.1}", iname, oname, total_d / count as f64);
            }
        }
    }
}
