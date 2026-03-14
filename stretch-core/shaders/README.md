# WGSL Compute Shaders — Stretch V4.2

GPU compute shaders for wgpu, loaded via `include_str!()` in `gpu.rs`.

| Shader | Workgroup | Description |
|--------|-----------|-------------|
| `plasticity.wgsl` | 256, 1 thread/edge | STDP ψ + eligibility + three-factor + homeostasis + consolidation |
| `propagation.wgsl` | 256, 1 thread/node | CSR inner loop: accumulate weighted source contributions |
| `budget.wgsl` | 256, 1 thread/edge | Two-pass budget normalization (atomic sum + scale) |

## Data layout

- **GpuEdge** (32 bytes): `from_node:u32, to_node:u32, conductance:f32, eligibility:f32, consolidated:u32, consolidation_counter:u32, distance:f32, _pad:f32`
- **GpuParams** (96 bytes): uniform buffer with all simulation parameters (see `gpu.rs`)
- All f64 values from CPU are cast to f32 for GPU compute.

## Reusable components

| Component | Module | Reusability |
|-----------|--------|-------------|
| IncomingCSR / OutgoingCSR | `domain.rs` | Any sparse graph (GNN, transport) |
| GpuContext | `gpu.rs` | Any wgpu compute pipeline |
| PerfMonitor | `perf.rs` | Any multi-phase pipeline |
| Three-factor learning | `stdp.rs` | Any RL neural network |
| ZoneManager + PID | `zone.rs` | Spatial PID control |
| KD-tree spatial | `domain.rs` | 3D graph with proximity queries |
