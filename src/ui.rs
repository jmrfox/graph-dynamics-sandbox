use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::dynamics::{Diffusion, Kuramoto, Wave};
use crate::palette::node_color;
use crate::sim::{Simulation, Topology};

pub fn draw_controls(ui: &mut Ui, sim: &mut Simulation) {
    if ui.button("Reset Defaults").clicked() {
        sim.dt = 0.01;
        sim.node_count = 6;
        sim.edge_prob = 0.4;
        sim.grid_rows = 4;
        sim.grid_cols = 4;
        sim.dynamics.reset_params();
    }

    ui.separator();
    ui.heading("Simulation");
    ui.separator();

    ui.horizontal(|ui| {
        let playing = !sim.paused;
        if ui.add(egui::Button::new("▶").selected(playing)).clicked() {
            sim.paused = false;
        }
        if ui.add(egui::Button::new("⏸").selected(sim.paused)).clicked() {
            sim.paused = true;
        }
        if ui.button("+dt").clicked() {
            let dt = sim.dt;
            sim.step(dt);
        }
        if ui.button("⏮").clicked() {
            sim.restart();
        }
    });

    ui.separator();
    ui.label("Timestep (dt)");
    ui.add(egui::Slider::new(&mut sim.dt, 0.001..=0.1).logarithmic(true));

    ui.separator();
    ui.label(format!("Time:  {:.3}", sim.time));
    ui.label(format!("Nodes: {}", sim.graph.node_count()));
    ui.label(format!("Edges: {}", sim.graph.edge_count()));
    ui.label(format!(
        "{}: {:.4}",
        if sim.dynamics.name() == "Kuramoto" { "Order R" } else { "Energy" },
        sim.dynamics.energy(&sim.nodes)
    ));

    ui.separator();
    ui.heading("System Options");
    ui.separator();
    draw_system_options(ui, sim);
}

fn draw_system_options(ui: &mut Ui, sim: &mut Simulation) {
    ui.label("Dynamics");

    let current = sim.dynamics.name().to_string();
    let dynamics_names = ["Diffusion", "Wave", "Kuramoto"];
    let mut new_dynamics: Option<Box<dyn crate::dynamics::Dynamics>> = None;

    for &name in &dynamics_names {
        if ui.selectable_label(current == name, name).clicked() && current != name {
            new_dynamics = Some(match name {
                "Wave" => Box::new(Wave::new()),
                "Kuramoto" => {
                    let rng = SmallRng::from_os_rng();
                    Box::new(Kuramoto::new(rng))
                }
                _ => Box::new(Diffusion::new()),
            });
        }
    }

    if let Some(dyn_box) = new_dynamics {
        sim.set_dynamics(dyn_box);
    } else {
        sim.dynamics.draw_controls(ui);
    }

    ui.separator();
    ui.label("Topology");

    let mut rebuild = false;

    let topo_names: &[(&str, Topology)] = &[
        ("Random", Topology::Random),
        ("Chain", Topology::Chain),
        ("Ring", Topology::Ring),
        ("Grid", Topology::Grid(sim.grid_rows, sim.grid_cols)),
        ("Small-world", Topology::SmallWorld),
        ("Scale-free", Topology::ScaleFree),
    ];

    for (label, topo) in topo_names {
        let selected = std::mem::discriminant(&sim.topology) == std::mem::discriminant(topo);
        if ui.selectable_label(selected, *label).clicked() && !selected {
            sim.topology = topo.clone();
            rebuild = true;
        }
    }

    if matches!(sim.topology, Topology::Grid(_, _)) {
        ui.horizontal(|ui| {
            ui.label("Rows");
            if ui
                .add(egui::DragValue::new(&mut sim.grid_rows).range(2..=12))
                .changed()
            {
                sim.topology = Topology::Grid(sim.grid_rows, sim.grid_cols);
                rebuild = true;
            }
            ui.label("Cols");
            if ui
                .add(egui::DragValue::new(&mut sim.grid_cols).range(2..=12))
                .changed()
            {
                sim.topology = Topology::Grid(sim.grid_rows, sim.grid_cols);
                rebuild = true;
            }
        });
    }

    if !matches!(sim.topology, Topology::Grid(_, _)) {
        ui.horizontal(|ui| {
            ui.label("Nodes");
            let old = sim.node_count;
            ui.add(egui::DragValue::new(&mut sim.node_count).range(2..=16));
            if sim.node_count != old {
                rebuild = true;
            }
        });
    }

    if matches!(sim.topology, Topology::Random) {
        ui.add(egui::Slider::new(&mut sim.edge_prob, 0.05..=1.0).text("edge prob"));
    }

    let can_randomize = matches!(
        sim.topology,
        Topology::Random | Topology::SmallWorld | Topology::ScaleFree
    );
    if can_randomize && ui.button("Randomize").clicked() {
        sim.randomize();
    }

    if rebuild {
        sim.rebuild_topology();
    }
}

pub fn draw_analysis(ui: &mut Ui, sim: &mut Simulation) {
    let dyn_name = sim.dynamics.name().to_string();
    sim.excitation.draw_panel(ui, &dyn_name);

    if sim.dampening.draw_panel(ui, &dyn_name) {
        let t = sim.time;
        sim.dampening.fire(t);
    }

    ui.heading("Analysis");
    ui.separator();

    let is_kuramoto = sim.dynamics.name() == "Kuramoto";

    if is_kuramoto {
        ui.label("Order parameter R(t)");
        let order_points: PlotPoints = sim
            .analysis
            .order_history
            .iter()
            .map(|&(t, r)| [t as f64, r as f64])
            .collect();
        Plot::new("order_plot")
            .height(240.0)
            .include_y(0.0)
            .include_y(1.0)
            .allow_zoom(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("R", order_points));
            });
    } else {
        ui.label("Energy E(t)");
        let energy_points: PlotPoints = sim
            .analysis
            .energy_history
            .iter()
            .map(|&(t, e)| [t as f64, e as f64])
            .collect();
        Plot::new("energy_plot")
            .height(240.0)
            .allow_zoom(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Energy", energy_points));
            });
    }

    ui.separator();

    let n = sim.graph.node_count();

    if is_kuramoto {
        ui.label("Instantaneous phases");

        let unit_circle: PlotPoints = (0..=64)
            .map(|k| {
                let a = k as f64 * std::f64::consts::TAU / 64.0;
                [a.cos(), a.sin()]
            })
            .collect();

        let phase_data: Vec<(usize, f64, f64)> = (0..n)
            .filter_map(|i| {
                sim.nodes.get(i).map(|node| {
                    let theta = node.s(0) as f64;
                    (i, theta.cos(), theta.sin())
                })
            })
            .collect();

        Plot::new("polar_plot")
            .height(240.0)
            .data_aspect(1.0)
            .include_x(-1.4)
            .include_x(1.4)
            .include_y(-1.4)
            .include_y(1.4)
            .allow_zoom(false)
            .allow_scroll(false)
            .show_axes([false, false])
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("circle", unit_circle)
                        .color(egui::Color32::from_gray(80)),
                );
                for (i, cx, cy) in &phase_data {
                    let color = node_color(*i, n);
                    plot_ui.points(
                        Points::new(format!("N{}", i), vec![[*cx, *cy]])
                            .color(color)
                            .radius(6.0),
                    );
                }
            });
    } else {
        ui.label("x(t) — activity");

        egui::Grid::new("node_sel")
            .num_columns(4)
            .spacing([6.0, 4.0])
            .show(ui, |ui| {
                for i in 0..n {
                    if i < sim.analysis.selected_nodes.len() {
                        let color = node_color(i, n);
                        let (r, g, b, _) = color.to_tuple();
                        ui.checkbox(&mut sim.analysis.selected_nodes[i], "");
                        ui.colored_label(
                            egui::Color32::from_rgb(r, g, b),
                            format!("N{}", i),
                        );
                        if (i + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                }
            });

        let node_histories: Vec<(usize, Vec<[f64; 2]>)> = sim
            .analysis
            .selected_nodes
            .iter()
            .enumerate()
            .filter(|&(_, sel)| *sel)
            .filter_map(|(i, _)| {
                sim.analysis.node_histories.get(i).map(|hist| {
                    let len = hist.len();
                    let points: Vec<[f64; 2]> = hist
                        .iter()
                        .enumerate()
                        .map(|(j, &v)| [j as f64 - len as f64, v as f64])
                        .collect();
                    (i, points)
                })
            })
            .collect();

        Plot::new("node_plot")
            .height(240.0)
            .allow_zoom(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                for (i, points) in node_histories {
                    plot_ui.line(
                        Line::new(format!("N{}", i), PlotPoints::new(points))
                            .color(node_color(i, n)),
                    );
                }
            });
    }
}
