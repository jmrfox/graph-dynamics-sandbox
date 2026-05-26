use std::f32::consts::TAU;

use egui::Color32;
use petgraph::graph::UnGraph;
use petgraph::visit::EdgeRef;
use rand::Rng;
use rand::rngs::SmallRng;

use crate::node::NodeState;

pub trait Dynamics {
    fn step(&mut self, graph: &UnGraph<(), f32>, nodes: &mut [NodeState], dt: f32);
    fn init_nodes(&self, n: usize, rng: &mut SmallRng) -> Vec<NodeState>;
    fn reset_nodes(&self, nodes: &mut [NodeState]);
    fn display_value(&self, node: &NodeState) -> f32;
    fn display_color(&self, node: &NodeState) -> Color32;
    fn draw_controls(&mut self, ui: &mut egui::Ui);
    fn name(&self) -> &str;
    fn energy(&self, nodes: &[NodeState]) -> f32;
    fn uses_cyclic_color(&self) -> bool {
        false
    }
    fn reset_params(&mut self) {}
}

// ── Diffusion ────────────────────────────────────────────────────────────────
// slot[0] = x

pub struct Diffusion {
    pub coeff: f32,
}

impl Diffusion {
    pub fn new() -> Self {
        Self { coeff: 0.5 }
    }
}

impl Dynamics for Diffusion {
    fn step(&mut self, graph: &UnGraph<(), f32>, nodes: &mut [NodeState], dt: f32) {
        let deltas: Vec<f32> = graph
            .node_indices()
            .map(|i| {
                let xi = nodes[i.index()].s(0);
                graph
                    .edges(i)
                    .map(|e| {
                        let w = *e.weight();
                        let xj = nodes[e.target().index()].s(0);
                        w * (xj - xi)
                    })
                    .sum::<f32>()
                    * self.coeff
                    * dt
            })
            .collect();
        for (i, d) in deltas.into_iter().enumerate() {
            *nodes[i].s_mut(0) += d;
        }
    }

    fn init_nodes(&self, n: usize, _rng: &mut SmallRng) -> Vec<NodeState> {
        let mut nodes = vec![NodeState::zero(); n];
        if n > 0 {
            *nodes[0].s_mut(0) = 1.0;
        }
        nodes
    }

    fn reset_nodes(&self, nodes: &mut [NodeState]) {
        for n in nodes.iter_mut() {
            n.slots = [0.0; 4];
        }
        if let Some(first) = nodes.first_mut() {
            *first.s_mut(0) = 1.0;
        }
    }

    fn display_value(&self, node: &NodeState) -> f32 {
        node.s(0)
    }

    fn display_color(&self, node: &NodeState) -> Color32 {
        gradient_color(node.s(0))
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.coeff, 0.0..=1.0).text("coeff"));
    }

    fn name(&self) -> &str {
        "Diffusion"
    }

    fn energy(&self, nodes: &[NodeState]) -> f32 {
        nodes.iter().map(|n| n.s(0) * n.s(0)).sum()
    }

    fn reset_params(&mut self) {
        self.coeff = 0.5;
    }
}

// ── Wave ─────────────────────────────────────────────────────────────────────
// slot[0] = x, slot[1] = v

pub struct Wave {
    pub speed: f32,
    pub damping: f32,
}

impl Wave {
    pub fn new() -> Self {
        Self { speed: 1.0, damping: 0.1 }
    }
}

impl Dynamics for Wave {
    fn step(&mut self, graph: &UnGraph<(), f32>, nodes: &mut [NodeState], dt: f32) {
        let c2 = self.speed * self.speed;
        let updates: Vec<(f32, f32)> = graph
            .node_indices()
            .map(|i| {
                let idx = i.index();
                let xi = nodes[idx].s(0);
                let vi = nodes[idx].s(1);
                let laplacian: f32 = graph
                    .edges(i)
                    .map(|e| {
                        let w = *e.weight();
                        let xj = nodes[e.target().index()].s(0);
                        w * (xj - xi)
                    })
                    .sum();
                let a = c2 * laplacian - self.damping * vi;
                let new_v = vi + a * dt;
                let new_x = xi + new_v * dt;
                (new_x, new_v)
            })
            .collect();
        for (i, (new_x, new_v)) in updates.into_iter().enumerate() {
            *nodes[i].s_mut(0) = new_x;
            *nodes[i].s_mut(1) = new_v;
        }
    }

    fn init_nodes(&self, n: usize, _rng: &mut SmallRng) -> Vec<NodeState> {
        let mut nodes = vec![NodeState::zero(); n];
        if n > 0 {
            *nodes[0].s_mut(0) = 1.0;
        }
        nodes
    }

    fn reset_nodes(&self, nodes: &mut [NodeState]) {
        for n in nodes.iter_mut() {
            n.slots = [0.0; 4];
        }
        if let Some(first) = nodes.first_mut() {
            *first.s_mut(0) = 1.0;
        }
    }

    fn display_value(&self, node: &NodeState) -> f32 {
        node.s(0)
    }

    fn display_color(&self, node: &NodeState) -> Color32 {
        gradient_color(node.s(0))
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.speed, 0.1..=3.0).text("speed"));
        ui.add(egui::Slider::new(&mut self.damping, 0.0..=1.0).text("damping"));
    }

    fn name(&self) -> &str {
        "Wave"
    }

    fn reset_params(&mut self) {
        self.speed = 1.0;
        self.damping = 0.1;
    }

    fn energy(&self, nodes: &[NodeState]) -> f32 {
        nodes
            .iter()
            .map(|n| n.s(0) * n.s(0) + n.s(1) * n.s(1))
            .sum()
    }
}

// ── Kuramoto ─────────────────────────────────────────────────────────────────
// slot[0] = theta, slot[1] = omega (intrinsic frequency, fixed per node)

pub struct Kuramoto {
    pub coupling: f32,
    pub freq_spread: f32,
    pub noise_sigma: f32,
    rng: SmallRng,
}

impl Kuramoto {
    pub fn new(rng: SmallRng) -> Self {
        Self {
            coupling: 0.5,
            freq_spread: 0.5,
            noise_sigma: 0.0,
            rng,
        }
    }
}

impl Dynamics for Kuramoto {
    fn step(&mut self, graph: &UnGraph<(), f32>, nodes: &mut [NodeState], dt: f32) {
        let deltas: Vec<f32> = graph
            .node_indices()
            .map(|i| {
                let idx = i.index();
                let theta_i = nodes[idx].s(0);
                let omega_i = nodes[idx].s(1);
                let coupling_sum: f32 = graph
                    .edges(i)
                    .map(|e| {
                        let w = *e.weight();
                        let theta_j = nodes[e.target().index()].s(0);
                        w * (theta_j - theta_i).sin()
                    })
                    .sum();
                omega_i + self.coupling * coupling_sum
            })
            .collect();

        let sigma_dt = self.noise_sigma * dt.sqrt();
        for (i, dtheta) in deltas.into_iter().enumerate() {
            let noise = if sigma_dt > 0.0 {
                self.rng.random::<f32>() * 2.0 * sigma_dt - sigma_dt
            } else {
                0.0
            };
            *nodes[i].s_mut(0) += (dtheta * dt + noise).rem_euclid(TAU);
            // keep theta in [0, TAU)
            let theta = nodes[i].s(0);
            *nodes[i].s_mut(0) = theta.rem_euclid(TAU);
        }
    }

    fn init_nodes(&self, n: usize, rng: &mut SmallRng) -> Vec<NodeState> {
        (0..n)
            .map(|_| {
                let mut node = NodeState::zero();
                *node.s_mut(0) = rng.random::<f32>() * TAU;
                *node.s_mut(1) = (rng.random::<f32>() - 0.5) * 2.0 * self.freq_spread;
                node
            })
            .collect()
    }

    fn reset_nodes(&self, nodes: &mut [NodeState]) {
        for n in nodes.iter_mut() {
            n.slots = [0.0; 4];
        }
    }

    fn display_value(&self, node: &NodeState) -> f32 {
        node.s(0) / TAU
    }

    fn display_color(&self, node: &NodeState) -> Color32 {
        hsv_color(node.s(0))
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.coupling, 0.0..=1.5).text("coupling K"));
        ui.add(egui::Slider::new(&mut self.freq_spread, 0.0..=1.0).text("freq spread σ"));
        ui.add(egui::Slider::new(&mut self.noise_sigma, 0.0..=1.0).text("noise σ"));
    }

    fn name(&self) -> &str {
        "Kuramoto"
    }

    fn uses_cyclic_color(&self) -> bool {
        true
    }

    fn reset_params(&mut self) {
        self.coupling = 0.5;
        self.freq_spread = 0.5;
        self.noise_sigma = 0.0;
    }

    fn energy(&self, nodes: &[NodeState]) -> f32 {
        // Order parameter R as "energy" proxy
        let n = nodes.len() as f32;
        if n == 0.0 {
            return 0.0;
        }
        let (sin_sum, cos_sum) = nodes.iter().fold((0.0f32, 0.0f32), |(s, c), node| {
            let theta = node.s(0);
            (s + theta.sin(), c + theta.cos())
        });
        ((sin_sum / n).powi(2) + (cos_sum / n).powi(2)).sqrt()
    }
}

pub fn order_parameter(nodes: &[NodeState]) -> f32 {
    let n = nodes.len() as f32;
    if n == 0.0 {
        return 0.0;
    }
    let (sin_sum, cos_sum) = nodes.iter().fold((0.0f32, 0.0f32), |(s, c), node| {
        let theta = node.s(0);
        (s + theta.sin(), c + theta.cos())
    });
    ((sin_sum / n).powi(2) + (cos_sum / n).powi(2)).sqrt()
}

// ── Color helpers ─────────────────────────────────────────────────────────────

pub fn gradient_color(v: f32) -> Color32 {
    let v = v.clamp(0.0, 1.0);
    let r = (v * 220.0) as u8;
    let g = ((1.0 - v) * 80.0 + v * 40.0) as u8;
    let b = ((1.0 - v) * 200.0) as u8;
    Color32::from_rgb(r, g, b)
}

pub fn hsv_color(theta: f32) -> Color32 {
    let hue = theta.rem_euclid(TAU) / TAU;
    let (r, g, b) = hsv_to_rgb(hue, 1.0, 0.9);
    Color32::from_rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor() as u32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
