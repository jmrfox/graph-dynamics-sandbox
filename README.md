# Graph Dynamics Sandbox

An interactive desktop sandbox for exploring dynamical systems on graphs. Pick a network topology, choose a model (diffusion, waves, or coupled oscillators), tune parameters in real time, and watch how activity propagates across nodes—with live plots for energy, order parameters, and per-node traces.

Built in Rust with [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) / [egui](https://github.com/emilk/egui) for the UI and [petgraph](https://docs.rs/petgraph) for graph structure.

## Features

- **Three dynamics models** on the same graph engine:
  - **Diffusion** — scalar state relaxes toward neighbors (graph Laplacian flow).
  - **Wave** — second-order wave equation with optional damping on edges.
  - **Kuramoto** — phase oscillators with coupling, frequency heterogeneity, and optional noise; includes the Kuramoto order parameter **R**.
- **Six topology generators**: random (Erdős–Rényi–style), chain, ring, grid, Watts–Strogatz small-world, and Barabási–Albert–style scale-free.
- **Weighted undirected edges** (random graphs use random weights in \([0.1, 1.0]\); structured topologies use unit weight).
- **Click-to-excite** any node with a configurable Gaussian pulse (amplitude and time scale).
- **Global dampening** pulse to suppress activity across all nodes.
- **Analysis panel**: energy or **R(t)** history, polar phase plot (Kuramoto), and selectable node time series (diffusion/wave).
- **Playback controls**: play/pause, single-step, restart, adjustable **dt**, and visualization scale for non-cyclic models.

## Screenshots

The main window has three regions:

| Left | Center | Right |
|------|--------|-------|
| Simulation and system controls | Graph visualization (nodes on a circle) | Excitation, dampening, and analysis plots |

Node fill color encodes the current state; the colored ring around each node identifies its index (**N0**, **N1**, …).

## Requirements

- [Rust](https://rustup.rs/) toolchain with **edition 2024** support (Rust **1.85+** recommended).
- A desktop environment with GPU/OpenGL support (standard for eframe/egui apps).

On **Windows**, **macOS**, and **Linux**, dependencies are pulled automatically by Cargo on first build.

## Quick start

Clone the repository and run:

```bash
cargo run --release
```

For a faster UI, prefer `--release`; debug builds are usable but slower.

The binary crate is named `gds` (see `Cargo.toml`). After building:

```bash
cargo build --release
# Windows
.\target\release\gds.exe
# macOS / Linux
./target/release/gds
```

## Using the app

### Simulation (left panel)

| Control | Action |
|---------|--------|
| **▶ / ⏸** | Run or pause the integrator |
| **+dt** | Advance one timestep while paused |
| **⏮** | Reset state to initial conditions (clears excitation/dampening pulses) |
| **Timestep (dt)** | Integration step size (log scale) |
| **Reset Defaults** | Restore default **dt**, node count, edge probability, grid size, and model parameters |

### Dynamics

Switch models with the selectable list; each exposes its own sliders:

| Model | State variables | Parameters |
|-------|-----------------|------------|
| **Diffusion** | \(x_i\) | `coeff` — diffusion strength |
| **Wave** | \(x_i\), \(v_i\) | `speed`, `damping` |
| **Kuramoto** | \(\theta_i\), \(\omega_i\) (fixed per node) | `coupling K`, `freq spread σ`, `noise σ` |

**Initial conditions**

- Diffusion / wave: a unit impulse on node **N0**, rest zero.
- Kuramoto: random phases on \([0, 2\pi)\) and intrinsic frequencies drawn from \([-\sigma, \sigma]\).

### Topology

| Topology | Notes |
|----------|--------|
| **Random** | **Nodes**, **edge prob**; **Randomize** draws a new graph |
| **Chain** | Path graph |
| **Ring** | Cycle |
| **Grid** | **Rows** / **Cols** (2–12); node count = rows × cols |
| **Small-world** | Watts–Strogatz-style rewiring (fixed \(k=4\), \(\beta=0.3\) in code) |
| **Scale-free** | Preferential attachment (fixed \(m=2\) in code) |

Changing topology rebuilds the graph and re-initializes node state.

### Graph view (center)

- Nodes are laid out on a circle; edges are drawn as segments.
- **Click a node** to apply the current excitation pulse (see right panel).
- For diffusion and wave, use the **scale** slider (top-left of the graph) to adjust how values map to color.

### Analysis (right panel)

- **Node Excitation** — pulse amplitude and duration; click nodes on the graph to fire.
- **Global Dampening** — **Dampen** applies a short global suppression pulse.
- **Diffusion / Wave** — **Energy E(t)** and per-node **x(t)** traces (toggle nodes in the grid).
- **Kuramoto** — **Order parameter R(t)** and a unit-circle **phase** plot.

## Models (brief)

**Diffusion** (explicit Euler on graph Laplacian):

\[
\dot{x}_i = \alpha \sum_{j \sim i} w_{ij}\,(x_j - x_i)
\]

**Wave** (discrete wave equation with velocity and damping):

\[
\dot{x}_i = v_i,\quad \dot{v}_i = c^2 \sum_{j \sim i} w_{ij}\,(x_j - x_i) - \gamma v_i
\]

**Kuramoto**:

\[
\dot{\theta}_i = \omega_i + K \sum_{j \sim i} w_{ij}\,\sin(\theta_j - \theta_i) + \text{(optional noise)}
\]

The order parameter \(R = \left|\frac{1}{N}\sum_j e^{i\theta_j}\right|\) measures phase coherence (plotted for Kuramoto runs).

## Project structure

```
graph-dynamics-sandbox/
├── Cargo.toml          # Package `gds`, dependencies
└── src/
    ├── main.rs         # eframe app shell, layout
    ├── sim.rs          # Simulation loop, topology switching
    ├── dynamics.rs     # Dynamics trait; Diffusion, Wave, Kuramoto
    ├── graph.rs        # Topology builders (petgraph)
    ├── node.rs         # Per-node state slots (up to 4 floats)
    ├── render.rs       # Graph drawing and click hit-testing
    ├── ui.rs           # Control and analysis panels
    ├── analysis.rs     # Time-series buffers for plots
    ├── excitation.rs   # Node excitation and global dampening
    └── palette.rs      # Consistent node identity colors
```

## Extending the sandbox

New dynamical systems implement the `Dynamics` trait in `dynamics.rs`:

- `step` — one integration step given the graph and `dt`
- `init_nodes` / `reset_nodes` — initial and restart state
- `display_value` / `display_color` — visualization
- `draw_controls` — parameter UI
- `name`, `energy`, and optionally `uses_cyclic_color` (e.g. phase on a hue wheel)

Register the model in `ui.rs` (`draw_system_options`) and wire excitation targets in `excitation.rs` if clicks should drive a specific state slot.

New topologies belong in `graph.rs` and in the `Topology` enum / `rebuild_topology` path in `sim.rs`.

## Dependencies

| Crate | Role |
|-------|------|
| `eframe`, `egui`, `egui_plot` | Window, widgets, plots |
| `petgraph` | `UnGraph` storage and traversal |
| `glam` | Math utilities (available for future layout/geometry) |
| `rand` | Stochastic graphs and Kuramoto noise |
| `serde` / `serde_json` | Serialization (available for save/load extensions) |
| `anyhow` | Error handling |

## License

[MIT](LICENSE) — Copyright (c) 2026 Jordan M. R. Fox, PhD.
