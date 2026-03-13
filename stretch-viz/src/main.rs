use macroquad::prelude::*;

use stretch_core::config::SimConfig;
use stretch_core::simulation::Simulation;

// ---------------------------------------------------------------------------
// Constantes de rendu
// ---------------------------------------------------------------------------
const SIDEBAR_W: f32 = 260.0;
const TOP_BAR_H: f32 = 40.0;
const PANEL_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Mapping couleur : valeur [0,1] → palette chaleur
// ---------------------------------------------------------------------------
fn heat_color(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if t < 0.2 {
        let s = t / 0.2;
        Color::new(0.0, 0.0, s, 1.0)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        Color::new(0.0, s, 1.0, 1.0)
    } else if t < 0.6 {
        let s = (t - 0.4) / 0.2;
        Color::new(s, 1.0, 1.0 - s, 1.0)
    } else if t < 0.8 {
        let s = (t - 0.6) / 0.2;
        Color::new(1.0, 1.0 - s, 0.0, 1.0)
    } else {
        let s = (t - 0.8) / 0.2;
        Color::new(1.0, s, s, 1.0)
    }
}

// ---------------------------------------------------------------------------
// Projection isométrique 3D → 2D
// ---------------------------------------------------------------------------
fn project_3d(pos: &[f64; 3], angle_y: f32, angle_x: f32) -> (f32, f32) {
    let (x, y, z) = (pos[0] as f32, pos[1] as f32, pos[2] as f32);
    // Rotation autour de Y (horizontal)
    let cos_y = angle_y.cos();
    let sin_y = angle_y.sin();
    let xr = x * cos_y + z * sin_y;
    let zr = -x * sin_y + z * cos_y;
    // Rotation autour de X (vertical)
    let cos_x = angle_x.cos();
    let sin_x = angle_x.sin();
    let yr = y * cos_x - zr * sin_x;
    // Orthographic projection: (xr, yr)
    (xr, yr)
}

// ---------------------------------------------------------------------------
// État de la visualisation
// ---------------------------------------------------------------------------
enum ViewMode {
    Activation,
    MemoryTrace,
    Fatigue,
    Conductance,
}

struct VizState {
    sim: Simulation,
    paused: bool,
    ticks_per_frame: usize,
    view_mode: ViewMode,
    is_grid: bool,
    grid_side: usize,
    finished: bool,
    // 3D rotation
    angle_y: f32,
    angle_x: f32,
}

impl VizState {
    fn new(config: SimConfig) -> Self {
        let is_grid = config.domain.topology == "grid2d";
        let grid_side = if is_grid { config.domain.size } else { 0 };
        VizState {
            sim: Simulation::new(config),
            paused: true,
            ticks_per_frame: 1,
            view_mode: ViewMode::Activation,
            is_grid,
            grid_side,
            finished: false,
            angle_y: 0.6,
            angle_x: 0.4,
        }
    }
}

// ---------------------------------------------------------------------------
// Point d'entrée macroquad
// ---------------------------------------------------------------------------
fn window_conf() -> Conf {
    Conf {
        window_title: "Stretch V1 — Visualisation 3D".to_string(),
        window_width: 1280,
        window_height: 900,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = if args.len() > 1 {
        let content = std::fs::read_to_string(&args[1])
            .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", args[1], e));
        toml::from_str::<SimConfig>(&content)
            .unwrap_or_else(|e| panic!("Erreur de parsing config : {}", e))
    } else {
        println!("Usage: stretch-viz <config.toml>");
        println!("Utilisation des valeurs par défaut.\n");
        SimConfig::default()
    };

    let mut viz = VizState::new(config);

    loop {
        // --- Entrées clavier ---
        if is_key_pressed(KeyCode::Space) {
            viz.paused = !viz.paused;
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::N) {
            if !viz.finished {
                viz.sim.step();
                if viz.sim.finished {
                    viz.finished = true;
                }
            }
        }
        if is_key_pressed(KeyCode::Key1) {
            viz.view_mode = ViewMode::Activation;
        }
        if is_key_pressed(KeyCode::Key2) {
            viz.view_mode = ViewMode::MemoryTrace;
        }
        if is_key_pressed(KeyCode::Key3) {
            viz.view_mode = ViewMode::Fatigue;
        }
        if is_key_pressed(KeyCode::Key4) {
            viz.view_mode = ViewMode::Conductance;
        }
        if is_key_pressed(KeyCode::Up) {
            viz.ticks_per_frame = (viz.ticks_per_frame * 2).min(256);
        }
        if is_key_pressed(KeyCode::Down) {
            viz.ticks_per_frame = (viz.ticks_per_frame / 2).max(1);
        }
        if is_key_pressed(KeyCode::R) {
            let config = viz.sim.config.clone();
            viz = VizState::new(config);
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
            break;
        }
        // Rotation 3D
        if is_key_down(KeyCode::A) {
            viz.angle_y -= 0.03;
        }
        if is_key_down(KeyCode::D) {
            viz.angle_y += 0.03;
        }
        if is_key_down(KeyCode::W) {
            viz.angle_x -= 0.03;
        }
        if is_key_down(KeyCode::S) {
            viz.angle_x += 0.03;
        }

        // --- Avancer la simulation ---
        if !viz.paused && !viz.finished {
            for _ in 0..viz.ticks_per_frame {
                if viz.sim.finished {
                    viz.finished = true;
                    break;
                }
                viz.sim.step();
            }
        }

        // --- Rendu ---
        clear_background(Color::new(0.12, 0.12, 0.15, 1.0));
        draw_top_bar(&viz);
        if viz.is_grid {
            draw_grid(&viz);
        } else {
            draw_points_3d(&viz);
        }
        draw_sidebar(&viz);

        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// Barre supérieure : tick, état, vitesse
// ---------------------------------------------------------------------------
fn draw_top_bar(viz: &VizState) {
    let status = if viz.finished {
        "TERMINÉ"
    } else if viz.paused {
        "PAUSE"
    } else {
        "▶ RUN"
    };

    let mode_name = match viz.view_mode {
        ViewMode::Activation => "Activation",
        ViewMode::MemoryTrace => "Trace mémoire",
        ViewMode::Fatigue => "Fatigue",
        ViewMode::Conductance => "Conductance",
    };

    let topo = &viz.sim.config.domain.topology;
    let text = format!(
        "Tick: {}/{} | {} | {}x | {} | {} [1-4]",
        viz.sim.tick,
        viz.sim.total_ticks(),
        status,
        viz.ticks_per_frame,
        topo,
        mode_name
    );
    draw_text(&text, 10.0, 25.0, 20.0, WHITE);
}

// ---------------------------------------------------------------------------
// Grille heatmap principale
// ---------------------------------------------------------------------------
fn draw_grid(viz: &VizState) {
    let sw = screen_width();
    let sh = screen_height();

    let grid_area_w = sw - SIDEBAR_W - PANEL_GAP * 2.0;
    let grid_area_h = sh - TOP_BAR_H - PANEL_GAP * 2.0;

    let side = viz.grid_side;
    if side == 0 {
        return;
    }
    let cell_w = grid_area_w / side as f32;
    let cell_h = grid_area_h / side as f32;
    let cell_size = cell_w.min(cell_h);

    let ox = PANEL_GAP;
    let oy = TOP_BAR_H + PANEL_GAP;

    // Calculer min/max pour normalisation
    let (values, label_max): (Vec<f64>, &str) = match viz.view_mode {
        ViewMode::Activation => {
            let vals: Vec<f64> = viz.sim.domain.nodes.iter().map(|n| n.activation).collect();
            (vals, "act")
        }
        ViewMode::MemoryTrace => {
            let vals: Vec<f64> = viz.sim.domain.nodes.iter().map(|n| n.memory_trace).collect();
            (vals, "trace")
        }
        ViewMode::Fatigue => {
            let vals: Vec<f64> = viz.sim.domain.nodes.iter().map(|n| n.fatigue).collect();
            (vals, "fatigue")
        }
        ViewMode::Conductance => {
            // Conductance moyenne des liaisons sortantes par nœud
            let mut cond_by_node = vec![0.0_f64; viz.sim.domain.nodes.len()];
            let mut count_by_node = vec![0_usize; viz.sim.domain.nodes.len()];
            for edge in &viz.sim.domain.edges {
                cond_by_node[edge.from] += edge.conductance;
                count_by_node[edge.from] += 1;
            }
            for i in 0..cond_by_node.len() {
                if count_by_node[i] > 0 {
                    cond_by_node[i] /= count_by_node[i] as f64;
                }
            }
            (cond_by_node, "cond")
        }
    };

    let max_val = values.iter().cloned().fold(0.001_f64, f64::max);

    for (i, &val) in values.iter().enumerate() {
        let row = i / side;
        let col = i % side;
        let normalized = (val / max_val) as f32;
        let color = heat_color(normalized);

        let x = ox + col as f32 * cell_size;
        let y = oy + row as f32 * cell_size;
        draw_rectangle(x, y, cell_size - 1.0, cell_size - 1.0, color);
    }

    // Légende
    let legend_y = oy + side as f32 * cell_size + 5.0;
    draw_text(
        &format!("{}: 0", label_max),
        ox,
        legend_y + 15.0,
        16.0,
        GRAY,
    );
    // Barre de couleur
    let bar_x = ox + 80.0;
    let bar_w = 200.0;
    for px in 0..bar_w as u32 {
        let t = px as f32 / bar_w;
        let c = heat_color(t);
        draw_line(bar_x + px as f32, legend_y + 3.0, bar_x + px as f32, legend_y + 13.0, 1.0, c);
    }
    draw_text(
        &format!("{:.2}", max_val),
        bar_x + bar_w + 5.0,
        legend_y + 15.0,
        16.0,
        GRAY,
    );
}

// ---------------------------------------------------------------------------
// Rendu 3D : nuage de points avec projection orthographique + rotation
// ---------------------------------------------------------------------------
fn draw_points_3d(viz: &VizState) {
    let sw = screen_width();
    let sh = screen_height();

    let area_w = sw - SIDEBAR_W - PANEL_GAP * 2.0;
    let area_h = sh - TOP_BAR_H - PANEL_GAP * 2.0;

    // Calculer les valeurs par nœud
    let (values, label_max): (Vec<f64>, &str) = match viz.view_mode {
        ViewMode::Activation => {
            (viz.sim.domain.nodes.iter().map(|n| n.activation).collect(), "act")
        }
        ViewMode::MemoryTrace => {
            (viz.sim.domain.nodes.iter().map(|n| n.memory_trace).collect(), "trace")
        }
        ViewMode::Fatigue => {
            (viz.sim.domain.nodes.iter().map(|n| n.fatigue).collect(), "fatigue")
        }
        ViewMode::Conductance => {
            let mut cond_by_node = vec![0.0_f64; viz.sim.domain.nodes.len()];
            let mut count_by_node = vec![0_usize; viz.sim.domain.nodes.len()];
            for edge in &viz.sim.domain.edges {
                cond_by_node[edge.from] += edge.conductance;
                count_by_node[edge.from] += 1;
            }
            for i in 0..cond_by_node.len() {
                if count_by_node[i] > 0 {
                    cond_by_node[i] /= count_by_node[i] as f64;
                }
            }
            (cond_by_node, "cond")
        }
    };

    let max_val = values.iter().cloned().fold(0.001_f64, f64::max);

    // Project all 3D positions to 2D
    let positions = &viz.sim.domain.positions;
    if positions.is_empty() {
        return;
    }

    // Compute bounding box of projected points for fitting
    let projected: Vec<(f32, f32)> = positions
        .iter()
        .map(|p| project_3d(p, viz.angle_y, viz.angle_x))
        .collect();

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for &(px, py) in &projected {
        min_x = min_x.min(px);
        max_x = max_x.max(px);
        min_y = min_y.min(py);
        max_y = max_y.max(py);
    }
    let range_x = (max_x - min_x).max(1.0);
    let range_y = (max_y - min_y).max(1.0);

    let margin = 20.0;
    let usable_w = area_w - margin * 2.0;
    let usable_h = area_h - margin * 2.0;
    let scale = (usable_w / range_x).min(usable_h / range_y);

    let ox = PANEL_GAP + margin + (usable_w - range_x * scale) * 0.5;
    let oy = TOP_BAR_H + PANEL_GAP + margin + (usable_h - range_y * scale) * 0.5;

    // Choose point radius based on number of nodes
    let n = positions.len();
    let point_r = if n < 500 { 4.0 } else if n < 5000 { 2.5 } else { 1.5 };

    for (i, &(px, py)) in projected.iter().enumerate() {
        let sx = ox + (px - min_x) * scale;
        let sy = oy + (py - min_y) * scale;
        let normalized = (values[i] / max_val) as f32;
        let color = if normalized < 0.01 {
            Color::new(0.15, 0.15, 0.18, 1.0) // faint for inactive
        } else {
            heat_color(normalized)
        };
        draw_circle(sx, sy, point_r, color);
    }

    // Légende
    let legend_y = TOP_BAR_H + PANEL_GAP + area_h - 25.0;
    draw_text(
        &format!("{}: 0", label_max),
        PANEL_GAP + 10.0,
        legend_y + 15.0,
        16.0,
        GRAY,
    );
    let bar_x = PANEL_GAP + 90.0;
    let bar_w = 200.0;
    for px in 0..bar_w as u32 {
        let t = px as f32 / bar_w;
        let c = heat_color(t);
        draw_line(bar_x + px as f32, legend_y + 3.0, bar_x + px as f32, legend_y + 13.0, 1.0, c);
    }
    draw_text(
        &format!("{:.2}", max_val),
        bar_x + bar_w + 5.0,
        legend_y + 15.0,
        16.0,
        GRAY,
    );

    // 3D rotation hint
    draw_text(
        "Rotation: W/A/S/D",
        PANEL_GAP + 10.0,
        legend_y - 5.0,
        14.0,
        DARKGRAY,
    );
}

// ---------------------------------------------------------------------------
// Sidebar : métriques en temps réel
// ---------------------------------------------------------------------------
fn draw_sidebar(viz: &VizState) {
    let sw = screen_width();
    let sx = sw - SIDEBAR_W;
    let sy = TOP_BAR_H + PANEL_GAP;

    // Fond sidebar
    draw_rectangle(sx, sy, SIDEBAR_W, screen_height() - sy, Color::new(0.08, 0.08, 0.10, 1.0));

    let mut y = sy + 20.0;
    let x = sx + 10.0;
    let line_h = 18.0;

    let draw_label = |y: &mut f32, label: &str, value: &str| {
        draw_text(label, x, *y, 15.0, GRAY);
        draw_text(value, x + 130.0, *y, 15.0, WHITE);
        *y += line_h;
    };

    draw_text("--- MÉTRIQUES ---", x, y, 16.0, YELLOW);
    y += line_h * 1.5;

    let nodes = &viz.sim.domain.nodes;
    let edges = &viz.sim.domain.edges;

    let active = nodes.iter().filter(|n| n.is_active()).count();
    let energy: f64 = nodes.iter().map(|n| n.activation).sum();
    let max_act = nodes.iter().map(|n| n.activation).fold(0.0_f64, f64::max);
    let mean_trace = nodes.iter().map(|n| n.memory_trace).sum::<f64>() / nodes.len().max(1) as f64;
    let max_trace = nodes.iter().map(|n| n.memory_trace).fold(0.0_f64, f64::max);
    let mean_fatigue = nodes.iter().map(|n| n.fatigue).sum::<f64>() / nodes.len().max(1) as f64;
    let mean_cond = edges.iter().map(|e| e.conductance).sum::<f64>() / edges.len().max(1) as f64;
    let max_cond = edges.iter().map(|e| e.conductance).fold(0.0_f64, f64::max);

    draw_label(&mut y, "Nœuds actifs:", &format!("{}", active));
    draw_label(&mut y, "Énergie:", &format!("{:.2}", energy));
    draw_label(&mut y, "Max activation:", &format!("{:.3}", max_act));
    y += line_h * 0.5;
    draw_label(&mut y, "Trace moyenne:", &format!("{:.3}", mean_trace));
    draw_label(&mut y, "Trace max:", &format!("{:.3}", max_trace));
    y += line_h * 0.5;
    draw_label(&mut y, "Fatigue moy.:", &format!("{:.3}", mean_fatigue));
    y += line_h * 0.5;
    draw_label(&mut y, "Cond. moyenne:", &format!("{:.3}", mean_cond));
    draw_label(&mut y, "Cond. max:", &format!("{:.3}", max_cond));

    y += line_h * 1.5;
    draw_text("--- CONTRÔLES ---", x, y, 16.0, YELLOW);
    y += line_h * 1.5;

    let controls = [
        ("ESPACE", "Pause / Play"),
        ("→ ou N", "1 tick (pas à pas)"),
        ("↑ / ↓", "Vitesse ×2 / ÷2"),
        ("1-2-3-4", "Vue: Act/Trace/Fat/Cond"),
        ("W/A/S/D", "Rotation 3D"),
        ("R", "Reset simulation"),
        ("Q / ESC", "Quitter"),
    ];
    for (key, desc) in &controls {
        draw_text(key, x, y, 14.0, GREEN);
        draw_text(desc, x + 75.0, y, 14.0, LIGHTGRAY);
        y += line_h;
    }

    // Mini sparkline énergie (dernières métriques)
    y += line_h;
    draw_text("--- ÉNERGIE ---", x, y, 16.0, YELLOW);
    y += line_h;
    let snapshots = &viz.sim.metrics.snapshots;
    if snapshots.len() > 1 {
        let n_points = snapshots.len().min(80);
        let start = snapshots.len() - n_points;
        let slice = &snapshots[start..];
        let max_e = slice
            .iter()
            .map(|s| s.global_energy)
            .fold(1.0_f64, f64::max);
        let sparkline_w = SIDEBAR_W - 20.0;
        let sparkline_h = 60.0;
        draw_rectangle_lines(x, y, sparkline_w, sparkline_h, 1.0, DARKGRAY);
        for i in 1..slice.len() {
            let x0 = x + (i - 1) as f32 / n_points as f32 * sparkline_w;
            let x1 = x + i as f32 / n_points as f32 * sparkline_w;
            let y0 = y + sparkline_h - (slice[i - 1].global_energy / max_e) as f32 * sparkline_h;
            let y1 = y + sparkline_h - (slice[i].global_energy / max_e) as f32 * sparkline_h;
            draw_line(x0, y0, x1, y1, 1.5, GREEN);
        }
    }
}
