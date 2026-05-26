use egui::{Color32, Painter, Pos2, Rect, Response, Stroke, Vec2};
use petgraph::graph::UnGraph;
use petgraph::visit::EdgeRef;
use std::f32::consts::{FRAC_PI_2, TAU};

use crate::dynamics::Dynamics;
use crate::node::NodeState;
use crate::palette::node_color;

pub fn draw_graph(
    painter: &Painter,
    response: &Response,
    dynamics: &dyn Dynamics,
    nodes: &[NodeState],
    graph: &UnGraph<(), f32>,
    rect: Rect,
    viz_scale: f32,
) -> Option<usize> {
    let n = graph.node_count();
    if n == 0 || nodes.len() < n {
        return None;
    }

    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.38;
    let node_radius = (radius / (n as f32).sqrt()).clamp(6.0, 22.0);
    let label_offset = node_radius + 6.0;

    let angles: Vec<f32> = (0..n)
        .map(|i| TAU * i as f32 / n as f32 - FRAC_PI_2)
        .collect();

    let positions: Vec<Pos2> = angles
        .iter()
        .map(|&a| center + Vec2::new(a.cos(), a.sin()) * radius)
        .collect();

    for edge in graph.edge_references() {
        let src = edge.source().index();
        let dst = edge.target().index();
        if src < positions.len() && dst < positions.len() {
            painter.line_segment(
                [positions[src], positions[dst]],
                Stroke::new(1.5, Color32::from_gray(100)),
            );
        }
    }

    let cyclic = dynamics.uses_cyclic_color();

    for i in 0..n {
        let node = &nodes[i];
        let raw_val = dynamics.display_value(node);

        let fill_color = if cyclic {
            dynamics.display_color(node)
        } else {
            let scaled_val = raw_val / viz_scale.max(1e-6);
            let mut scaled_node = node.clone();
            *scaled_node.s_mut(0) = scaled_val;
            dynamics.display_color(&scaled_node)
        };

        let identity = node_color(i, n);

        painter.circle_filled(positions[i], node_radius, fill_color);
        painter.circle_stroke(positions[i], node_radius, Stroke::new(3.5, identity));

        let val_label = format!("{:.2}", raw_val);
        painter.text(
            positions[i],
            egui::Align2::CENTER_CENTER,
            val_label,
            egui::FontId::proportional((node_radius * 0.7).max(9.0)),
            Color32::WHITE,
        );

        let label_pos = center + Vec2::new(angles[i].cos(), angles[i].sin()) * (radius + label_offset);
        let align = angle_to_align(angles[i]);
        painter.text(
            label_pos,
            align,
            format!("N{}", i),
            egui::FontId::proportional(11.0),
            Color32::from_gray(210),
        );
    }

    if response.clicked() {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            for (i, &pos) in positions.iter().enumerate() {
                if (pointer_pos - pos).length() <= node_radius {
                    return Some(i);
                }
            }
        }
    }

    None
}

fn angle_to_align(angle: f32) -> egui::Align2 {
    let x = angle.cos();
    let y = angle.sin();
    let ax = x.abs();
    let ay = y.abs();
    if ax > ay {
        if x > 0.0 { egui::Align2::LEFT_CENTER } else { egui::Align2::RIGHT_CENTER }
    } else {
        if y > 0.0 { egui::Align2::CENTER_TOP } else { egui::Align2::CENTER_BOTTOM }
    }
}
