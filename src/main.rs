mod analysis;
mod dynamics;
mod excitation;
mod graph;
mod node;
mod palette;
mod render;
mod sim;
mod ui;

use eframe::egui;

use dynamics::order_parameter;
use render::draw_graph;
use sim::Simulation;
use ui::{draw_analysis, draw_controls};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_title("Graph Dynamics Sandbox"),
        ..Default::default()
    };

    eframe::run_native(
        "Graph Dynamics Sandbox",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

struct App {
    sim: Simulation,
    viz_scale: f32,
}

impl App {
    fn new() -> Self {
        Self {
            sim: Simulation::new(),
            viz_scale: 1.0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.sim.paused {
            let dt = self.sim.dt;
            self.sim.step(dt);

            if self.sim.dynamics.name() == "Kuramoto" {
                let r = order_parameter(&self.sim.nodes);
                self.sim.analysis.push_order(self.sim.time, r);
            }

            ctx.request_repaint();
        }

        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(220.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    draw_controls(ui, &mut self.sim);
                });
            });

        egui::SidePanel::right("analysis")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    draw_analysis(ui, &mut self.sim);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, egui::Sense::click());
            let painter = ui.painter();
            if let Some(node_idx) = draw_graph(
                painter,
                &response,
                &*self.sim.dynamics,
                &self.sim.nodes,
                &self.sim.graph,
                rect,
                self.viz_scale,
            ) {
                self.sim.fire_excitation(node_idx);
            }

            if !self.sim.dynamics.uses_cyclic_color() {
                let slider_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(8.0, 8.0),
                    egui::vec2(160.0, 20.0),
                );
                ui.put(
                    slider_rect,
                    egui::Slider::new(&mut self.viz_scale, 0.01..=10.0)
                        .logarithmic(true)
                        .text("scale"),
                );
            }
        });
    }
}
