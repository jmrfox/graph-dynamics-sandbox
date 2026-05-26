use egui::Ui;

use crate::node::NodeState;

#[derive(Clone, Copy, PartialEq)]
pub enum ExcitationTarget {
    DiffusionX,
    WaveV,
    KuramotoFreq,
}

impl ExcitationTarget {
    pub fn slot(&self) -> usize {
        match self {
            ExcitationTarget::DiffusionX => 0,
            ExcitationTarget::WaveV => 1,
            ExcitationTarget::KuramotoFreq => 1,
        }
    }
}

struct ActiveExcitation {
    node_idx: usize,
    t_start: f32,
    slot: usize,
}

pub struct ExcitationManager {
    pub amplitude: f32,
    pub sigma: f32,
    pub target: ExcitationTarget,
    active: Vec<ActiveExcitation>,
}

impl ExcitationManager {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            sigma: 0.1,
            target: ExcitationTarget::DiffusionX,
            active: Vec::new(),
        }
    }

    pub fn fire(&mut self, node_idx: usize, sim_time: f32) {
        self.active.push(ActiveExcitation {
            node_idx,
            t_start: sim_time,
            slot: self.target.slot(),
        });
    }

    pub fn apply(&mut self, nodes: &mut [NodeState], sim_time: f32, dt: f32) {
        let sigma = self.sigma;
        let amplitude = self.amplitude;
        let cutoff = 6.0 * sigma;
        let center = 3.0 * sigma;

        self.active.retain(|exc| {
            let elapsed = sim_time - exc.t_start;
            if elapsed > cutoff {
                return false;
            }
            if let Some(node) = nodes.get_mut(exc.node_idx) {
                let t = elapsed - center;
                let g = (amplitude / sigma) * (-0.5 * (t / sigma).powi(2)).exp();
                *node.s_mut(exc.slot) += g * dt;
            }
            true
        });
    }

    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn set_dynamics(&mut self, dynamics_name: &str) {
        self.target = match dynamics_name {
            "Wave" => ExcitationTarget::WaveV,
            "Kuramoto" => ExcitationTarget::KuramotoFreq,
            _ => ExcitationTarget::DiffusionX,
        };
        self.clear();
    }

    pub fn draw_panel(&mut self, ui: &mut Ui, _dynamics_name: &str) {
        ui.heading("Node Excitation (click)");
        ui.separator();

        ui.add(
            egui::Slider::new(&mut self.amplitude, -1.0..=1.0)
                .text("amplitude"),
        );
        ui.add(
            egui::Slider::new(&mut self.sigma, 0.01..=10.0)
                .logarithmic(true)
                .text("time scale (s)"),
        );
    }
}

// ── Dampening ─────────────────────────────────────────────────────────────────

struct ActiveDampening {
    t_start: f32,
    slot: usize,
}

pub struct DampeningManager {
    pub amplitude: f32,
    pub sigma: f32,
    pub target: ExcitationTarget,
    active: Vec<ActiveDampening>,
}

impl DampeningManager {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            sigma: 0.1,
            target: ExcitationTarget::DiffusionX,
            active: Vec::new(),
        }
    }

    pub fn fire(&mut self, sim_time: f32) {
        self.active.push(ActiveDampening {
            t_start: sim_time,
            slot: self.target.slot(),
        });
    }

    pub fn apply(&mut self, nodes: &mut [NodeState], sim_time: f32, dt: f32) {
        let sigma = self.sigma;
        let amplitude = self.amplitude;
        let cutoff = 6.0 * sigma;
        let center = 3.0 * sigma;

        self.active.retain(|dmp| {
            let elapsed = sim_time - dmp.t_start;
            if elapsed > cutoff {
                return false;
            }
            let t = elapsed - center;
            let g = (amplitude / sigma) * (-0.5 * (t / sigma).powi(2)).exp();
            let factor = (1.0 - g * dt).max(0.0);
            for node in nodes.iter_mut() {
                *node.s_mut(dmp.slot) *= factor;
            }
            true
        });
    }

    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn set_dynamics(&mut self, dynamics_name: &str) {
        self.target = match dynamics_name {
            "Wave" => ExcitationTarget::WaveV,
            "Kuramoto" => ExcitationTarget::KuramotoFreq,
            _ => ExcitationTarget::DiffusionX,
        };
        self.clear();
    }

    pub fn draw_panel(&mut self, ui: &mut Ui, _dynamics_name: &str) -> bool {
        ui.heading("Global Dampening");
        ui.separator();
        let clicked = ui.button("Dampen").clicked();

        ui.add(
            egui::Slider::new(&mut self.amplitude, 0.0..=1.0)
                .text("amplitude"),
        );
        ui.add(
            egui::Slider::new(&mut self.sigma, 0.01..=10.0)
                .logarithmic(true)
                .text("time scale (s)"),
        );

        clicked
    }
}
