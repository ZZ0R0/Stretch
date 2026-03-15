use macroquad::prelude::*;
use std::sync::Arc;

use stretch_core::config::{SimConfig, V5TaskMode, V5BaselineMode};
use stretch_core::diagnostics::{self, TracedPath};
use stretch_core::simulation::{Simulation, VizSnapshot};

// ---------------------------------------------------------------------------
// Rendering constants
// ---------------------------------------------------------------------------
const SIDEBAR_W: f32 = 280.0;
const TOP_BAR_H: f32 = 40.0;
const PANEL_GAP: f32 = 8.0;
const MAX_TOP_EDGES: usize = 500;

// I/O group colors
const IO_COLORS: [Color; 5] = [
    Color::new(0.0, 0.0, 0.0, 0.0),   // 0: not I/O (unused)
    Color::new(0.2, 0.6, 1.0, 1.0),   // 1: input-0 (cyan)
    Color::new(0.9, 0.3, 0.9, 1.0),   // 2: input-1 (magenta)
    Color::new(0.2, 1.0, 0.4, 1.0),   // 3: output-0 (green)
    Color::new(1.0, 0.7, 0.1, 1.0),   // 4: output-1 (orange)
];
const PATH_COLORS: [Color; 4] = [
    Color::new(0.2, 1.0, 0.4, 0.9),   // class 0: green
    Color::new(1.0, 0.5, 0.1, 0.9),   // class 1: orange
    Color::new(0.3, 0.7, 1.0, 0.9),   // class 2 (future)
    Color::new(1.0, 0.3, 0.3, 0.9),   // class 3 (future)
];

// ---------------------------------------------------------------------------
// Color palette: value [0,1] → cold-to-hot
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
// 3D → 2D isometric projection
// ---------------------------------------------------------------------------
fn project_3d(pos: &[f64; 3], angle_y: f32, angle_x: f32) -> (f32, f32) {
    let (x, y, z) = (pos[0] as f32, pos[1] as f32, pos[2] as f32);
    let cos_y = angle_y.cos();
    let sin_y = angle_y.sin();
    let xr = x * cos_y + z * sin_y;
    let zr = -x * sin_y + z * cos_y;
    let cos_x = angle_x.cos();
    let sin_x = angle_x.sin();
    let yr = y * cos_x - zr * sin_x;
    (xr, yr)
}

// ---------------------------------------------------------------------------
// View modes
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Activation,
    MemoryTrace,
    Fatigue,
    Conductance,
}

// ---------------------------------------------------------------------------
// Projection context (reused by overlay renderers)
// ---------------------------------------------------------------------------
struct ProjCtx {
    projected: Vec<(f32, f32)>,
    min_x: f32,
    min_y: f32,
    scale: f32,
    ox: f32,
    oy: f32,
}

impl ProjCtx {
    fn screen_xy(&self, i: usize) -> (f32, f32) {
        let (px, py) = self.projected[i];
        (self.ox + (px - self.min_x) * self.scale,
         self.oy + (py - self.min_y) * self.scale)
    }
}

// ---------------------------------------------------------------------------
// Visualization state
// ---------------------------------------------------------------------------
struct VizState {
    sim: Simulation,
    paused: bool,
    ticks_per_frame: usize,
    view_mode: ViewMode,
    is_grid: bool,
    grid_side: usize,
    finished: bool,
    angle_y: f32,
    angle_x: f32,
    positions: Arc<Vec<[f64; 3]>>,
    is_excitatory: Arc<Vec<bool>>,
    topology: String,
    // ── V5 ──
    is_v5: bool,
    input_groups: Vec<Vec<usize>>,
    output_groups: Vec<Vec<usize>>,
    target_mapping: Vec<usize>,
    task_label: String,
    baseline_label: String,
    /// Per-node I/O type: 0=regular, 1=in0, 2=in1, 3=out0, 4=out1
    io_map: Vec<u8>,
    // ── Overlay toggles ──
    show_io: bool,
    show_paths: bool,
    show_edges: bool,
    // ── Cached diagnostics (recomputed on demand with T key) ──
    cached_paths: Vec<Option<TracedPath>>,
    cached_top_edges: Vec<(usize, usize, f32)>, // (from_node, to_node, conductance)
    cached_node_cond: Vec<f32>,
    /// Nodes on ≥2 traced paths (co-reinforcement clusters)
    cached_cluster_nodes: Vec<usize>,
    diag_computed: bool,
    // ── Accuracy tracking ──
    accuracy_history: Vec<f32>,
    last_evaluated: usize,
    // ── Conductance history (for timeline) ──
    conductance_history: Vec<f32>,
}

impl VizState {
    fn new(config: SimConfig) -> Self {
        let is_grid = config.domain.topology == "grid2d";
        let grid_side = if is_grid { config.domain.size } else { 0 };
        let topology = config.domain.topology.clone();
        let task_label = format!("{:?}", config.v5_task.task_mode);
        let baseline_label = format!("{:?}", config.v5_task.baseline_mode);

        let is_v5 = config.v5_task.task_mode != V5TaskMode::Legacy
            || config.v5_task.invert_mapping
            || config.v5_task.baseline_mode != V5BaselineMode::FullLearning;

        let mut sim = Simulation::new(config);
        let n = if is_v5 {
            sim.setup_v5_training()
        } else {
            sim.setup_v4_training()
        };
        eprintln!("[Viz {}] {} trials", if is_v5 { "V5" } else { "V4" }, n);

        let input_groups = sim.input_encoder.as_ref()
            .map(|e| e.groups.clone()).unwrap_or_default();
        let output_groups = sim.output_reader.as_ref()
            .map(|r| r.groups.clone()).unwrap_or_default();
        let target_mapping = sim.target_mapping.clone();

        let n_nodes = sim.domain.num_nodes();
        let positions = Arc::new(sim.domain.positions.clone());
        let is_excitatory = Arc::new(
            sim.domain.nodes.iter()
                .map(|n| n.node_type == stretch_core::node::NeuronType::Excitatory)
                .collect(),
        );

        // Build I/O lookup map
        let mut io_map = vec![0u8; n_nodes];
        for (g, nodes) in input_groups.iter().enumerate() {
            for &i in nodes {
                if i < n_nodes { io_map[i] = (g + 1) as u8; }
            }
        }
        for (g, nodes) in output_groups.iter().enumerate() {
            for &i in nodes {
                if i < n_nodes { io_map[i] = (g + input_groups.len() + 1) as u8; }
            }
        }

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
            is_v5,
            input_groups,
            output_groups,
            target_mapping,
            task_label,
            baseline_label,
            io_map,
            show_io: true,
            show_paths: false,
            show_edges: false,
            cached_paths: Vec::new(),
            cached_top_edges: Vec::new(),
            cached_node_cond: vec![0.0; n_nodes],
            cached_cluster_nodes: Vec::new(),
            diag_computed: false,
            accuracy_history: Vec::new(),
            last_evaluated: 0,
            conductance_history: Vec::new(),
        }
    }

    fn build_snapshot(&self) -> VizSnapshot {
        self.sim.build_viz_snapshot(&self.positions, &self.is_excitatory)
    }

    /// Recompute Dijkstra paths, top edges, and node conductances (on demand).
    fn refresh_diagnostics(&mut self) {
        let t0 = std::time::Instant::now();

        // Paths: for each input class, find best path to target output
        self.cached_paths.clear();
        for k in 0..self.input_groups.len() {
            let target_idx = if k < self.target_mapping.len() {
                self.target_mapping[k]
            } else { k };
            if target_idx < self.output_groups.len() {
                let path = diagnostics::trace_best_path(
                    &self.sim.domain,
                    &self.input_groups[k],
                    &self.output_groups[target_idx],
                    0.05,
                );
                self.cached_paths.push(path);
            } else {
                self.cached_paths.push(None);
            }
        }

        // Top edges by conductance deviation from 1.0
        let domain = &self.sim.domain;
        let mut devs: Vec<(usize, f32)> = domain.edges.iter()
            .enumerate()
            .map(|(i, e)| (i, (e.conductance - 1.0).abs()))
            .filter(|(_, d)| *d > 0.01)
            .collect();
        devs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        self.cached_top_edges.clear();
        for &(ei, _) in devs.iter().take(MAX_TOP_EDGES) {
            let e = &domain.edges[ei];
            self.cached_top_edges.push((e.from, e.to, e.conductance));
        }

        // Node-level mean outgoing conductance (for conductance view mode)
        let n = domain.num_nodes();
        self.cached_node_cond.resize(n, 0.0);
        for i in 0..n {
            let start = domain.outgoing.offsets[i];
            let end = domain.outgoing.offsets[i + 1];
            if start < end {
                let sum: f32 = (start..end)
                    .map(|k| domain.edges[domain.outgoing.edge_indices[k]].conductance)
                    .sum();
                self.cached_node_cond[i] = sum / (end - start) as f32;
            } else {
                self.cached_node_cond[i] = 0.0;
            }
        }

        // Cluster analysis: nodes on ≥2 traced paths
        let mut visit_count = vec![0u8; n];
        for path_opt in &self.cached_paths {
            if let Some(p) = path_opt {
                for &node in &p.nodes {
                    visit_count[node] = visit_count[node].saturating_add(1);
                }
            }
        }
        self.cached_cluster_nodes = visit_count.iter()
            .enumerate()
            .filter(|(_, &c)| c >= 2)
            .map(|(i, _)| i)
            .collect();

        self.diag_computed = true;
        eprintln!("[Viz] Diagnostics: {} paths, {} top edges, {} cluster nodes ({:.0}ms)",
            self.cached_paths.iter().filter(|p| p.is_some()).count(),
            self.cached_top_edges.len(),
            self.cached_cluster_nodes.len(),
            t0.elapsed().as_secs_f64() * 1000.0);
    }

    fn track_accuracy(&mut self) {
        if self.sim.total_evaluated > self.last_evaluated {
            self.accuracy_history.push(self.sim.accuracy() as f32);
            self.last_evaluated = self.sim.total_evaluated;
        }
        // Conductance history (sample every 10 evaluations)
        if self.sim.total_evaluated > 0 && self.sim.total_evaluated % 10 == 0
            && self.conductance_history.len() < self.sim.total_evaluated / 10
        {
            if let Some(tm) = self.sim.metrics.snapshots.last() {
                self.conductance_history.push(tm.mean_conductance);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------
fn window_conf() -> Conf {
    Conf {
        window_title: "Stretch V5 — Visualisation 3D".to_string(),
        window_width: 1400,
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
            .unwrap_or_else(|e| panic!("Cannot read {} : {}", args[1], e));
        toml::from_str::<SimConfig>(&content)
            .unwrap_or_else(|e| panic!("Config parse error: {}", e))
    } else {
        println!("Usage: stretch-viz <config.toml>");
        println!("Using defaults.\n");
        SimConfig::default()
    };

    let mut viz = VizState::new(config);

    loop {
        // --- Keyboard ---
        if is_key_pressed(KeyCode::Space) { viz.paused = !viz.paused; }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::N) {
            if !viz.finished {
                viz.sim.step();
                if viz.sim.finished { viz.finished = true; }
            }
        }
        if is_key_pressed(KeyCode::Key1) { viz.view_mode = ViewMode::Activation; }
        if is_key_pressed(KeyCode::Key2) { viz.view_mode = ViewMode::MemoryTrace; }
        if is_key_pressed(KeyCode::Key3) { viz.view_mode = ViewMode::Fatigue; }
        if is_key_pressed(KeyCode::Key4) {
            viz.view_mode = ViewMode::Conductance;
            if !viz.diag_computed { viz.refresh_diagnostics(); }
        }
        if is_key_pressed(KeyCode::Up) { viz.ticks_per_frame = (viz.ticks_per_frame * 2).min(256); }
        if is_key_pressed(KeyCode::Down) { viz.ticks_per_frame = (viz.ticks_per_frame / 2).max(1); }

        // Overlay toggles
        if is_key_pressed(KeyCode::I) { viz.show_io = !viz.show_io; }
        if is_key_pressed(KeyCode::P) {
            viz.show_paths = !viz.show_paths;
            if viz.show_paths && !viz.diag_computed { viz.refresh_diagnostics(); }
        }
        if is_key_pressed(KeyCode::C) {
            viz.show_edges = !viz.show_edges;
            if viz.show_edges && !viz.diag_computed { viz.refresh_diagnostics(); }
        }
        if is_key_pressed(KeyCode::T) { viz.refresh_diagnostics(); }

        if is_key_pressed(KeyCode::R) {
            let config = viz.sim.config.clone();
            viz = VizState::new(config);
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) { break; }

        // 3D rotation
        if is_key_down(KeyCode::A) { viz.angle_y -= 0.03; }
        if is_key_down(KeyCode::D) { viz.angle_y += 0.03; }
        if is_key_down(KeyCode::W) { viz.angle_x -= 0.03; }
        if is_key_down(KeyCode::S) { viz.angle_x += 0.03; }

        // --- Advance simulation ---
        if !viz.paused && !viz.finished {
            for _ in 0..viz.ticks_per_frame {
                if viz.sim.finished { viz.finished = true; break; }
                viz.sim.step();
            }
        }

        // --- Track accuracy ---
        viz.track_accuracy();

        // --- Build snapshot ---
        let snap = viz.build_snapshot();

        // --- Render ---
        clear_background(Color::new(0.12, 0.12, 0.15, 1.0));
        draw_top_bar(&snap, &viz);
        if viz.is_grid {
            draw_grid(&snap, &viz);
        } else {
            draw_3d_view(&snap, &viz);
        }
        draw_sidebar(&snap, &viz);

        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// Top bar
// ---------------------------------------------------------------------------
fn draw_top_bar(snap: &VizSnapshot, viz: &VizState) {
    let status = if snap.finished { "DONE" }
        else if viz.paused { "PAUSE" }
        else { "▶ RUN" };

    let mode_name = match viz.view_mode {
        ViewMode::Activation => "Activation",
        ViewMode::MemoryTrace => "Trace",
        ViewMode::Fatigue => "Fatigue",
        ViewMode::Conductance => "Conductance",
    };

    let total = viz.sim.total_ticks();
    let tick_str = if total == 0 { format!("{}", snap.tick) }
        else { format!("{}/{}", snap.tick, total) };

    let v5_tag = if viz.is_v5 {
        format!(" | V5 {}/{}", viz.task_label, viz.baseline_label)
    } else { String::new() };

    let overlays = format!("{}{}{}",
        if viz.show_io { " [I/O]" } else { "" },
        if viz.show_paths { " [Paths]" } else { "" },
        if viz.show_edges { " [Edges]" } else { "" },
    );

    let text = format!(
        "Tick: {} | {} | {}x | {} | {} [1-4]{}{}",
        tick_str, status, viz.ticks_per_frame, &viz.topology, mode_name, v5_tag, overlays
    );
    draw_text(&text, 10.0, 25.0, 18.0, WHITE);
}

// ---------------------------------------------------------------------------
// Grid heatmap (unchanged, for grid2d topology)
// ---------------------------------------------------------------------------
fn draw_grid(snap: &VizSnapshot, viz: &VizState) {
    let sw = screen_width();
    let sh = screen_height();
    let grid_area_w = sw - SIDEBAR_W - PANEL_GAP * 2.0;
    let grid_area_h = sh - TOP_BAR_H - PANEL_GAP * 2.0;
    let side = viz.grid_side;
    if side == 0 { return; }
    let cell_w = grid_area_w / side as f32;
    let cell_h = grid_area_h / side as f32;
    let cell_size = cell_w.min(cell_h);
    let ox = PANEL_GAP;
    let oy = TOP_BAR_H + PANEL_GAP;

    let values: &[f32] = match viz.view_mode {
        ViewMode::Activation => &snap.activations,
        ViewMode::MemoryTrace => &snap.memory_traces,
        ViewMode::Fatigue => &snap.fatigues,
        ViewMode::Conductance => &viz.cached_node_cond,
    };
    let max_val = values.iter().cloned().fold(0.001_f32, f32::max);

    for (i, &val) in values.iter().enumerate() {
        let row = i / side;
        let col = i % side;
        let x = ox + col as f32 * cell_size;
        let y = oy + row as f32 * cell_size;
        draw_rectangle(x, y, cell_size - 1.0, cell_size - 1.0, heat_color(val / max_val));
    }
}

// ---------------------------------------------------------------------------
// 3D view with overlays
// ---------------------------------------------------------------------------
fn draw_3d_view(snap: &VizSnapshot, viz: &VizState) {
    let sw = screen_width();
    let sh = screen_height();
    let area_w = sw - SIDEBAR_W - PANEL_GAP * 2.0;
    let area_h = sh - TOP_BAR_H - PANEL_GAP * 2.0;

    let positions = &*snap.positions;
    if positions.is_empty() { return; }

    // Build projection context
    let projected: Vec<(f32, f32)> = positions.iter()
        .map(|p| project_3d(p, viz.angle_y, viz.angle_x))
        .collect();

    let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
    let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    for &(px, py) in &projected {
        min_x = min_x.min(px); max_x = max_x.max(px);
        min_y = min_y.min(py); max_y = max_y.max(py);
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
    let point_r = if n < 500 { 4.0 } else if n < 5000 { 2.5 } else { 1.2 };

    let ctx = ProjCtx { projected, min_x, min_y, scale, ox, oy };

    // Select values for coloring
    let (values, label): (&[f32], &str) = match viz.view_mode {
        ViewMode::Activation => (&snap.activations, "act"),
        ViewMode::MemoryTrace => (&snap.memory_traces, "trace"),
        ViewMode::Fatigue => (&snap.fatigues, "fatigue"),
        ViewMode::Conductance => (&viz.cached_node_cond, "cond"),
    };
    let max_val = values.iter().cloned().fold(0.001_f32, f32::max);

    // ── Layer 1: Edge overlay (behind everything) ──
    if viz.show_edges && !viz.cached_top_edges.is_empty() {
        let cond_max = viz.cached_top_edges.iter()
            .map(|&(_, _, c)| c)
            .fold(0.001_f32, f32::max);
        for &(from, to, cond) in &viz.cached_top_edges {
            if from >= n || to >= n { continue; }
            let (x1, y1) = ctx.screen_xy(from);
            let (x2, y2) = ctx.screen_xy(to);
            let t = (cond / cond_max).clamp(0.0, 1.0);
            let alpha = 0.15 + 0.5 * t;
            let color = Color::new(
                heat_color(t).r,
                heat_color(t).g,
                heat_color(t).b,
                alpha,
            );
            draw_line(x1, y1, x2, y2, 0.8, color);
        }
    }

    // ── Layer 2: Path overlay ──
    if viz.show_paths {
        for (class_idx, path_opt) in viz.cached_paths.iter().enumerate() {
            if let Some(path) = path_opt {
                let color = PATH_COLORS[class_idx % PATH_COLORS.len()];
                for w in path.nodes.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if a >= n || b >= n { continue; }
                    let (x1, y1) = ctx.screen_xy(a);
                    let (x2, y2) = ctx.screen_xy(b);
                    draw_line(x1, y1, x2, y2, 2.5, color);
                }
            }
        }
        // Cluster nodes (on ≥2 paths): bright yellow
        for &i in &viz.cached_cluster_nodes {
            if i >= n { continue; }
            let (sx, sy) = ctx.screen_xy(i);
            draw_circle(sx, sy, point_r * 2.5, YELLOW);
        }
    }

    // ── Layer 3: Inactive nodes (dim) ──
    let dim_exc = Color::new(0.15, 0.15, 0.18, 1.0);
    let dim_inh = Color::new(0.18, 0.10, 0.22, 1.0);
    let d = point_r * 2.0;
    for i in 0..n {
        if values.get(i).copied().unwrap_or(0.0) >= 0.01 { continue; }
        let (sx, sy) = ctx.screen_xy(i);
        let color = if snap.is_excitatory[i] { dim_exc } else { dim_inh };
        draw_rectangle(sx - point_r, sy - point_r, d, d, color);
    }

    // ── Layer 4: Active nodes (colored by view mode) ──
    for &i in &snap.active_indices {
        let (sx, sy) = ctx.screen_xy(i);
        let val = values.get(i).copied().unwrap_or(0.0);
        let normalized = val / max_val;
        let color = if !snap.is_excitatory[i] {
            let t = normalized.clamp(0.0, 1.0);
            Color::new(0.3 * t, 0.1 * t, 0.6 + 0.4 * t, 1.0)
        } else {
            heat_color(normalized)
        };
        draw_rectangle(sx - point_r, sy - point_r, d, d, color);
    }

    // ── Layer 5: I/O group markers (on top) ──
    if viz.show_io {
        let io_r = (point_r * 3.0).max(3.0);
        for (i, &io_type) in viz.io_map.iter().enumerate() {
            if io_type == 0 || i >= n { continue; }
            let (sx, sy) = ctx.screen_xy(i);
            let color = IO_COLORS[io_type as usize];
            draw_circle(sx, sy, io_r, color);
            draw_circle_lines(sx, sy, io_r + 1.0, 1.0, WHITE);
        }
    }

    // ── Legend ──
    let legend_y = TOP_BAR_H + PANEL_GAP + area_h - 30.0;

    // Color bar
    draw_text(&format!("{}: 0", label), PANEL_GAP + 10.0, legend_y + 15.0, 15.0, GRAY);
    let bar_x = PANEL_GAP + 80.0;
    let bar_w = 180.0;
    for px in 0..bar_w as u32 {
        let t = px as f32 / bar_w;
        let c = heat_color(t);
        draw_line(bar_x + px as f32, legend_y + 3.0, bar_x + px as f32, legend_y + 13.0, 1.0, c);
    }
    draw_text(&format!("{:.2}", max_val), bar_x + bar_w + 5.0, legend_y + 15.0, 15.0, GRAY);

    // I/O legend (if showing)
    if viz.show_io {
        let ly = legend_y - 22.0;
        let items: [(&str, Color); 4] = [
            ("In-0", IO_COLORS[1]), ("In-1", IO_COLORS[2]),
            ("Out-0", IO_COLORS[3]), ("Out-1", IO_COLORS[4]),
        ];
        let mut lx = PANEL_GAP + 10.0;
        for (name, color) in &items {
            draw_circle(lx + 5.0, ly - 3.0, 5.0, *color);
            draw_text(name, lx + 14.0, ly + 2.0, 14.0, LIGHTGRAY);
            lx += 75.0;
        }
    }

    // Path legend (if showing)
    if viz.show_paths && !viz.cached_paths.is_empty() {
        let ly = legend_y - 42.0;
        let mut lx = PANEL_GAP + 10.0;
        for (k, path_opt) in viz.cached_paths.iter().enumerate() {
            if let Some(p) = path_opt {
                let color = PATH_COLORS[k % PATH_COLORS.len()];
                draw_line(lx, ly - 2.0, lx + 20.0, ly - 2.0, 2.5, color);
                draw_text(
                    &format!("Path {} ({} hops, RS={:.1})", k, p.nodes.len() - 1, p.route_score),
                    lx + 25.0, ly + 2.0, 13.0, LIGHTGRAY,
                );
                lx += 200.0;
            }
        }
    }

    draw_text("Rotation: W/A/S/D", PANEL_GAP + 10.0, legend_y - 60.0, 13.0, DARKGRAY);
}

// ---------------------------------------------------------------------------
// Sidebar: real-time metrics + V5 info + accuracy sparkline
// ---------------------------------------------------------------------------
fn draw_sidebar(snap: &VizSnapshot, viz: &VizState) {
    let sw = screen_width();
    let sx = sw - SIDEBAR_W;
    let sy = TOP_BAR_H + PANEL_GAP;

    draw_rectangle(sx, sy, SIDEBAR_W, screen_height() - sy, Color::new(0.08, 0.08, 0.10, 1.0));

    let mut y = sy + 15.0;
    let x = sx + 10.0;
    let line_h = 16.0;

    let draw_label = |y: &mut f32, label: &str, value: &str| {
        draw_text(label, x, *y, 14.0, GRAY);
        draw_text(value, x + 120.0, *y, 14.0, WHITE);
        *y += line_h;
    };

    let m = &snap.metrics;

    // ── V5 Info (if applicable) ──
    if viz.is_v5 {
        draw_text("--- V5 INFO ---", x, y, 15.0, Color::new(0.3, 0.8, 1.0, 1.0));
        y += line_h * 1.2;
        draw_label(&mut y, "Task:", &viz.task_label);
        draw_label(&mut y, "Baseline:", &viz.baseline_label);
        let mapping_str: Vec<String> = viz.target_mapping.iter()
            .enumerate()
            .map(|(k, &t)| format!("{}→{}", k, t))
            .collect();
        draw_label(&mut y, "Mapping:", &mapping_str.join(", "));
        y += line_h * 0.3;
    }

    // ── Metrics ──
    draw_text("--- METRICS ---", x, y, 15.0, YELLOW);
    y += line_h * 1.2;
    draw_label(&mut y, "Active:", &format!("{}", m.active_count));
    draw_label(&mut y, "Energy:", &format!("{:.2}", m.total_energy));
    draw_label(&mut y, "Max act:", &format!("{:.3}", m.max_activation));
    draw_label(&mut y, "Mean trace:", &format!("{:.3}", m.mean_trace));
    draw_label(&mut y, "Mean cond:", &format!("{:.3}", m.mean_conductance));
    draw_label(&mut y, "Max cond:", &format!("{:.3}", m.max_conductance));
    if m.consolidated_edges > 0 {
        draw_label(&mut y, "Consol:", &format!("{}", m.consolidated_edges));
    }
    y += line_h * 0.3;

    // ── Training ──
    draw_text("--- TRAINING ---", x, y, 15.0, YELLOW);
    y += line_h * 1.2;
    draw_label(&mut y, "Trial:", &format!("{}/{}", m.current_trial, m.total_trials));
    draw_label(&mut y, "Dopamine:", &format!("{:.3}", m.dopamine_level));
    if m.total_evaluated > 0 {
        let acc = m.accuracy * 100.0;
        draw_label(&mut y, "Accuracy:", &format!("{:.1}% ({}/{})", acc, m.correct_count, m.total_evaluated));
    }
    if let Some(dec) = m.last_decision {
        draw_label(&mut y, "Decision:", &format!("{}", dec));
    }
    if let Some(tgt) = m.last_target {
        draw_label(&mut y, "Target:", &format!("{}", tgt));
    }

    // ── E/I balance ──
    if m.active_excitatory > 0 || m.active_inhibitory > 0 {
        y += line_h * 0.3;
        draw_label(&mut y, "Excit:", &format!("{}", m.active_excitatory));
        draw_label(&mut y, "Inhib:", &format!("{}", m.active_inhibitory));
    }

    // ── Diagnostics info (if computed) ──
    if viz.diag_computed && !viz.cached_paths.is_empty() {
        y += line_h * 0.3;
        draw_text("--- PATHS ---", x, y, 15.0, Color::new(0.3, 1.0, 0.5, 1.0));
        y += line_h * 1.2;
        for (k, path_opt) in viz.cached_paths.iter().enumerate() {
            match path_opt {
                Some(p) => {
                    let txt = format!("C{}: {} hops RS={:.2} BN={:.3}",
                        k, p.nodes.len() - 1, p.route_score, p.min_conductance);
                    draw_text(&txt, x, y, 13.0, LIGHTGRAY);
                }
                None => {
                    draw_text(&format!("C{}: no path", k), x, y, 13.0, RED);
                }
            }
            y += line_h;
        }
        if !viz.cached_cluster_nodes.is_empty() {
            draw_text(&format!("Cluster: {} shared nodes", viz.cached_cluster_nodes.len()),
                x, y, 13.0, YELLOW);
            y += line_h;
        }
    }

    // ── Controls ──
    y += line_h * 0.5;
    draw_text("--- CONTROLS ---", x, y, 15.0, YELLOW);
    y += line_h * 1.2;
    let controls = [
        ("SPACE", "Pause/Play"),
        ("→/N", "Step 1 tick"),
        ("↑/↓", "Speed ×2/÷2"),
        ("1-4", "Act/Trc/Fat/Cond"),
        ("WASD", "Rotate 3D"),
        ("I", "I/O groups"),
        ("P", "Paths overlay"),
        ("C", "Top edges"),
        ("T", "Refresh diag"),
        ("R", "Reset"),
        ("Q", "Quit"),
    ];
    for (key, desc) in &controls {
        draw_text(key, x, y, 13.0, GREEN);
        draw_text(desc, x + 55.0, y, 13.0, LIGHTGRAY);
        y += line_h;
    }

    // ── Accuracy sparkline ──
    if viz.accuracy_history.len() > 1 {
        y += line_h * 0.5;
        draw_text("--- ACCURACY ---", x, y, 15.0, YELLOW);
        y += line_h;
        let sparkline_w = SIDEBAR_W - 20.0;
        let sparkline_h = 50.0;
        draw_rectangle_lines(x, y, sparkline_w, sparkline_h, 1.0, DARKGRAY);
        // 50% baseline
        let baseline_y = y + sparkline_h * 0.5;
        draw_line(x, baseline_y, x + sparkline_w, baseline_y, 0.5, DARKGRAY);
        let n_pts = viz.accuracy_history.len();
        let start = if n_pts > 100 { n_pts - 100 } else { 0 };
        let slice = &viz.accuracy_history[start..];
        for i in 1..slice.len() {
            let x0 = x + (i - 1) as f32 / slice.len() as f32 * sparkline_w;
            let x1 = x + i as f32 / slice.len() as f32 * sparkline_w;
            let y0 = y + sparkline_h * (1.0 - slice[i - 1]);
            let y1 = y + sparkline_h * (1.0 - slice[i]);
            draw_line(x0, y0, x1, y1, 1.5, GREEN);
        }
        y += sparkline_h + 5.0;
    }

    // ── Energy sparkline ──
    if snap.energy_history.len() > 1 {
        draw_text("--- ENERGY ---", x, y, 15.0, YELLOW);
        y += line_h;
        let sparkline_w = SIDEBAR_W - 20.0;
        let sparkline_h = 50.0;
        draw_rectangle_lines(x, y, sparkline_w, sparkline_h, 1.0, DARKGRAY);
        let n_pts = snap.energy_history.len();
        let max_e = snap.energy_history.iter().cloned().fold(1.0_f32, f32::max);
        for i in 1..n_pts {
            let x0 = x + (i - 1) as f32 / n_pts as f32 * sparkline_w;
            let x1 = x + i as f32 / n_pts as f32 * sparkline_w;
            let y0 = y + sparkline_h - (snap.energy_history[i - 1] / max_e) * sparkline_h;
            let y1 = y + sparkline_h - (snap.energy_history[i] / max_e) * sparkline_h;
            draw_line(x0, y0, x1, y1, 1.5, Color::new(0.3, 0.8, 1.0, 1.0));
        }
        y += sparkline_h + 5.0;
    }

    // ── Conductance timeline ──
    if viz.conductance_history.len() > 1 {
        draw_text("--- COND TIMELINE ---", x, y, 15.0, YELLOW);
        y += line_h;
        let sparkline_w = SIDEBAR_W - 20.0;
        let sparkline_h = 40.0;
        draw_rectangle_lines(x, y, sparkline_w, sparkline_h, 1.0, DARKGRAY);
        let n_pts = viz.conductance_history.len();
        let max_c = viz.conductance_history.iter().cloned().fold(0.001_f32, f32::max);
        let min_c = viz.conductance_history.iter().cloned().fold(f32::MAX, f32::min);
        let range = (max_c - min_c).max(0.001);
        for i in 1..n_pts {
            let x0 = x + (i - 1) as f32 / n_pts as f32 * sparkline_w;
            let x1 = x + i as f32 / n_pts as f32 * sparkline_w;
            let y0 = y + sparkline_h * (1.0 - (viz.conductance_history[i - 1] - min_c) / range);
            let y1 = y + sparkline_h * (1.0 - (viz.conductance_history[i] - min_c) / range);
            draw_line(x0, y0, x1, y1, 1.5, Color::new(1.0, 0.6, 0.2, 1.0));
        }
    }
}
