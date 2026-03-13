mod config;
mod domain;
mod edge;
mod metrics;
mod node;
mod plasticity;
mod propagation;
mod simulation;
mod stimulus;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = if args.len() > 1 && args[1] != "--gen-config" {
        let config_path = &args[1];
        let content = fs::read_to_string(config_path)
            .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", config_path, e));
        toml::from_str::<config::SimConfig>(&content)
            .unwrap_or_else(|e| panic!("Erreur de parsing config : {}", e))
    } else {
        if args.len() > 1 && args[1] == "--gen-config" {
            let default_config = config::SimConfig::default();
            let toml_str =
                toml::to_string_pretty(&default_config).expect("Erreur de sérialisation TOML");
            let gen_path = "default_config.toml";
            fs::write(gen_path, &toml_str).expect("Impossible d'écrire la config");
            println!("Config par défaut générée : {}", gen_path);
            return;
        }
        println!("Aucun fichier de config fourni, utilisation des valeurs par défaut.");
        println!("Usage: stretch-v0 <config.toml>\n");
        config::SimConfig::default()
    };

    // Exécution
    let result = simulation::run(&config);

    // Export des métriques en JSON
    let output_path = "metrics_output.json";
    let json =
        serde_json::to_string_pretty(&result.metrics).expect("Erreur de sérialisation JSON");
    fs::write(output_path, &json).expect("Impossible d'écrire le fichier de métriques");
    println!("\nMétriques exportées dans : {}", output_path);

    // Export résumé des nœuds (traces mémoire)
    let traces_path = "node_traces.json";
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
    fs::write(traces_path, &traces_json).expect("Impossible d'écrire les traces");
    println!("Traces nœuds exportées dans : {}", traces_path);

    // Export résumé des liaisons modifiées
    let edges_path = "edge_conductances.json";
    let edge_data: Vec<serde_json::Value> = result
        .domain
        .edges
        .iter()
        .filter(|e| e.usage_count > 0)
        .map(|e| {
            serde_json::json!({
                "from": e.from,
                "to": e.to,
                "conductance": e.conductance,
                "usage_count": e.usage_count,
                "coactivity_trace": e.coactivity_trace
            })
        })
        .collect();
    let edges_json =
        serde_json::to_string_pretty(&edge_data).expect("Erreur de sérialisation edges");
    fs::write(edges_path, &edges_json).expect("Impossible d'écrire les conductances");
    println!("Conductances exportées dans : {}", edges_path);

    // Histogramme des traces
    let histogram = metrics::MetricsLog::trace_histogram(&result.domain, 10);
    println!("\nHistogramme des traces mémoire :");
    for (lower, count) in &histogram {
        let bar: String = std::iter::repeat('#').take(*count / 5 + 1).collect();
        println!("  [{:.3}, ...) : {:>5} {}", lower, count, bar);
    }
}
