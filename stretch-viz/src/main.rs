use macroquad::prelude::*;
use std::sync::Arc;

use stretch_core::config::SimConfig;
use stretch_core::simulation::{Simulation, VizSnapshot};

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
    // Shared constant data for snapshots
    positions: Arc<Vec<[f64; 3]>>,
    is_excitatory: Arc<Vec<bool>>,
    // Topology name (for top bar)
    topology: String,
}

impl VizState {
    fn new(config: SimConfig) -> Self {
        let is_grid = config.domain.topology == "grid2d";
        let grid_side = if is_grid { config.domain.size } else { 0 };
        let topology = config.domain.topology.clone();
        let mut sim = Simulation::new(config);
        // V4 : configurer I/O spatial + trials
        let n = sim.setup_v4_training();
        eprintln!("[Viz V4] {} trials programmés", n);

        let positions = Arc::new(sim.domain.positions.clone());
        let is_excitatory = Arc::new(
            sim.domain.nodes.iter()
                .map(|n| n.node_type == stretch_core::node::NeuronType::Excitatory)
                .collect(),
        );

        VizState {
            sim,
            paused: false,
            ticks_per_frame: 1,
            view_mode: ViewMode::Activation,
            is_grid,
            grid_side,
            finished: false,
            angle_y: 0.6,
            angle_x: 0.4,
            positions,
            is_excitatory,
            topology,
        }
    }

    fn build_snapshot(&self) -> VizSnapshot {
        self.sim.build_viz_snapshot(&self.positions, &self.is_excitatory)
    }
}

// ---------------------------------------------------------------------------
// Point d'entrée macroquad
// ---------------------------------------------------------------------------
fn window_conf() -> Conf {
    Conf {
        window_title: "Stretch V4 — Visualisation 3D".to_string(),
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

        // --- Build snapshot for rendering ---
        let snap = viz.build_snapshot();

        // --- Rendu ---
        clear_background(Color::new(0.12, 0.12, 0.15, 1.0));
        draw_top_bar(&snap, &viz);
        if viz.is_grid {
            draw_grid(&snap, &viz);
        } else {
            draw_points_3d(&snap, &viz);
        }
        draw_sidebar(&snap);

        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// Barre supérieure : tick, état, vitesse
// ---------------------------------------------------------------------------
fn draw_top_bar(snap: &VizSnapshot, viz: &VizState) {
    let status = if snap.finished {
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
    };

    let total = viz.sim.total_ticks();
    let tick_str = if total == 0 {
        format!("{}", snap.tick)
    } else {
        format!("{}/{}", snap.tick, total)
    };
    let text = format!(
        "Tick: {} | {} | {}x | {} | {} [1-3]",
        tick_str,
        status,
        viz.ticks_per_frame,
        &viz.topology,
        mode_name
    );
    draw_text(&text, 10.0, 25.0, 20.0, WHITE);
}

// ---------------------------------------------------------------------------
// Grille heatmap principale
// ---------------------------------------------------------------------------
fn draw_grid(snap: &VizSnapshot, viz: &VizState) {
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

    let (values, label_max): (&[f32], &str) = match viz.view_mode {
        ViewMode::Activation => (&snap.activations, "act"),
        ViewMode::MemoryTrace => (&snap.memory_traces, "trace"),
        ViewMode::Fatigue => (&snap.fatigues, "fatigue"),
    };

    let max_val = values.iter().cloned().fold(0.001_f32, f32::max);

    for (i, &val) in values.iter().enumerate() {
        let row = i / side;
        let col = i % side;
        let normalized = val / max_val;
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
fn draw_points_3d(snap: &VizSnapshot, viz: &VizState) {
    let sw = screen_width();
    let sh = screen_height();

    let area_w = sw - SIDEBAR_W - PANEL_GAP * 2.0;
    let area_h = sh - TOP_BAR_H - PANEL_GAP * 2.0;

    let values: &[f32] = match viz.view_mode {
        ViewMode::Activation => &snap.activations,
        ViewMode::MemoryTrace => &snap.memory_traces,
        ViewMode::Fatigue => &snap.fatigues,
    };
    let label_max = match viz.view_mode {
        ViewMode::Activation => "act",
        ViewMode::MemoryTrace => "trace",
        ViewMode::Fatigue => "fatigue",
    };

    let max_val = values.iter().cloned().fold(0.001_f32, f32::max);

    let positions = &*snap.positions;
    if positions.is_empty() {
        return;
    }

    // Project all 3D positions to 2D
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

    let margin_px = 20.0;
    let usable_w = area_w - margin_px * 2.0;
    let usable_h = area_h - margin_px * 2.0;
    let scale = (usable_w / range_x).min(usable_h / range_y);

    let ox = PANEL_GAP + margin_px + (usable_w - range_x * scale) * 0.5;
    let oy = TOP_BAR_H + PANEL_GAP + margin_px + (usable_h - range_y * scale) * 0.5;

    let n = positions.len();
    let point_r = if n < 500 { 4.0 } else if n < 5000 { 2.5 } else { 1.5 };
    let d = point_r * 2.0;

    // Culling: only draw active nodes (activation > 0.01)
    // Inactive nodes get a dim background color
    // We draw all nodes with a dim base, then overdraw active ones
    let dim_exc = Color::new(0.15, 0.15, 0.18, 1.0);
    let dim_inh = Color::new(0.18, 0.10, 0.22, 1.0);

    // Draw inactive nodes as dim background
    for (i, &(px, py)) in projected.iter().enumerate() {
        if values[i] >= 0.01 {
            continue; // will be drawn in active pass
        }
        let sx = ox + (px - min_x) * scale;
        let sy = oy + (py - min_y) * scale;
        let color = if snap.is_excitatory[i] { dim_exc } else { dim_inh };
        draw_rectangle(sx - point_r, sy - point_r, d, d, color);
    }

    // Draw active nodes with full color (using active_indices for culling)
    for &i in &snap.active_indices {
        let (px, py) = projected[i];
        let sx = ox + (px - min_x) * scale;
        let sy = oy + (py - min_y) * scale;
        let normalized = values[i] / max_val;
        let color = if !snap.is_excitatory[i] {
            let t = normalized.clamp(0.0, 1.0);
            Color::new(0.3 * t, 0.1 * t, 0.6 + 0.4 * t, 1.0)
        } else {
            heat_color(normalized)
        };
        draw_rectangle(sx - point_r, sy - point_r, d, d, color);
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
fn draw_sidebar(snap: &VizSnapshot) {
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

    let m = &snap.metrics;

    draw_text("--- MÉTRIQUES ---", x, y, 16.0, YELLOW);
    y += line_h * 1.5;

    draw_label(&mut y, "Nœuds actifs:", &format!("{}", m.active_count));
    draw_label(&mut y, "Énergie:", &format!("{:.2}", m.total_energy));
    draw_label(&mut y, "Max activation:", &format!("{:.3}", m.max_activation));
    y += line_h * 0.5;
    draw_label(&mut y, "Trace moyenne:", &format!("{:.3}", m.mean_trace));
    draw_label(&mut y, "Trace max:", &format!("{:.3}", m.max_trace));
    y += line_h * 0.5;
    draw_label(&mut y, "Fatigue moy.:", &format!("{:.3}", m.mean_fatigue));
    y += line_h * 0.5;
    draw_label(&mut y, "Cond. moyenne:", &format!("{:.3}", m.mean_conductance));
    draw_label(&mut y, "Cond. max:", &format!("{:.3}", m.max_conductance));

    if m.consolidated_edges > 0 {
        y += line_h * 0.5;
        draw_label(&mut y, "Consolidées:", &format!("{}", m.consolidated_edges));
    }

    if m.num_zones > 0 {
        y += line_h * 0.5;
        draw_text("--- ZONES PID ---", x, y, 16.0, YELLOW);
        y += line_h * 1.2;
        draw_label(&mut y, "Zones:", &format!("{}", m.num_zones));
        draw_label(&mut y, "Act. moy.:", &format!("{:.4}", m.zone_activity_mean));
        draw_label(&mut y, "Err. PID:", &format!("{:.4}", m.mean_pid_error));
        draw_label(&mut y, "Out. PID:", &format!("{:.4}", m.mean_pid_output));
        draw_label(&mut y, "Mode PID:", &m.pid_mode);
    }

    // E/I metrics
    if m.active_inhibitory > 0 || m.active_excitatory > 0 {
        y += line_h * 0.5;
        draw_text("--- E/I ---", x, y, 16.0, YELLOW);
        y += line_h * 1.2;
        draw_label(&mut y, "Excit. (actifs):", &format!("{}", m.active_excitatory));
        draw_label(&mut y, "Inhib. (actifs):", &format!("{}", m.active_inhibitory));
        draw_label(&mut y, "Énergie E:", &format!("{:.2}", m.excitatory_energy));
        draw_label(&mut y, "Énergie I:", &format!("{:.2}", m.inhibitory_energy));
    }

    // V4 dopamine / training
    {
        y += line_h * 0.5;
        draw_text("--- V4 DOPAMINE ---", x, y, 16.0, YELLOW);
        y += line_h * 1.2;
        draw_label(&mut y, "Dopa. level:", &format!("{:.3}", m.dopamine_level));
        draw_label(&mut y, "Trial:", &format!("{}/{}", m.current_trial, m.total_trials));
        if m.total_evaluated > 0 {
            let acc = m.accuracy * 100.0;
            draw_label(&mut y, "Accuracy:", &format!("{:.1}% ({}/{})", acc, m.correct_count, m.total_evaluated));
        }
        if let Some(dec) = m.last_decision {
            draw_label(&mut y, "Last decision:", &format!("{}", dec));
        }
        if let Some(tgt) = m.last_target {
            draw_label(&mut y, "Last target:", &format!("{}", tgt));
        }
    }

    y += line_h * 1.5;
    draw_text("--- CONTRÔLES ---", x, y, 16.0, YELLOW);
    y += line_h * 1.5;

    let controls = [
        ("ESPACE", "Pause / Play"),
        ("→ ou N", "1 tick (pas à pas)"),
        ("↑ / ↓", "Vitesse ×2 / ÷2"),
        ("1-2-3", "Vue: Act/Trace/Fat"),
        ("W/A/S/D", "Rotation 3D"),
        ("R", "Reset simulation"),
        ("Q / ESC", "Quitter"),
    ];
    for (key, desc) in &controls {
        draw_text(key, x, y, 14.0, GREEN);
        draw_text(desc, x + 75.0, y, 14.0, LIGHTGRAY);
        y += line_h;
    }

    // Energy sparkline
    y += line_h;
    draw_text("--- ÉNERGIE ---", x, y, 16.0, YELLOW);
    y += line_h;
    if snap.energy_history.len() > 1 {
        let n_points = snap.energy_history.len();
        let max_e = snap.energy_history.iter().cloned().fold(1.0_f32, f32::max);
        let sparkline_w = SIDEBAR_W - 20.0;
        let sparkline_h = 60.0;
        draw_rectangle_lines(x, y, sparkline_w, sparkline_h, 1.0, DARKGRAY);
        for i in 1..n_points {
            let x0 = x + (i - 1) as f32 / n_points as f32 * sparkline_w;
            let x1 = x + i as f32 / n_points as f32 * sparkline_w;
            let y0 = y + sparkline_h - (snap.energy_history[i - 1] / max_e) * sparkline_h;
            let y1 = y + sparkline_h - (snap.energy_history[i] / max_e) * sparkline_h;
            draw_line(x0, y0, x1, y1, 1.5, GREEN);
        }
    }
}
