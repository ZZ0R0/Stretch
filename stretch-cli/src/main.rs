use std::env;
use std::fs;

use stretch_core::config::SimConfig;
use stretch_core::metrics::MetricsLog;
use stretch_core::simulation::{self, Simulation, NullObserver};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = if args.len() > 1 && args[1] != "--gen-config" {
        let config_path = &args[1];
        let content = fs::read_to_string(config_path)
            .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", config_path, e));
        toml::from_str::<SimConfig>(&content)
            .unwrap_or_else(|e| panic!("Erreur de parsing config : {}", e))
    } else {
        if args.len() > 1 && args[1] == "--gen-config" {
            let default_config = SimConfig::default();
            let toml_str =
                toml::to_string_pretty(&default_config).expect("Erreur de sérialisation TOML");
            let gen_path = "default_config.toml";
            fs::write(gen_path, &toml_str).expect("Impossible d'écrire la config");
            println!("Config par défaut générée : {}", gen_path);
            return;
        }
        println!("Aucun fichier de config fourni, utilisation des valeurs par défaut.");
        println!("Usage: stretch-cli <config.toml>\n");
        SimConfig::default()
    };

    // V4 : lancement en mode entraînement avec trials, via run_with_observer
    let mut sim = Simulation::new(config.clone());
    let n_trials = sim.setup_v4_training();

    // Spatial diagnostics
    print_spatial_diagnostics(&sim);

    let trial_period = if n_trials > 1 {
        sim.trials[1].start_tick - sim.trials[0].start_tick
    } else { 0 };
    println!("[V4 Training] {} trials programmés ({} classes, période={})", n_trials, config.input.num_classes, trial_period);

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
