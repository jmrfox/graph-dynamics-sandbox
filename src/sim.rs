use petgraph::graph::UnGraph;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::analysis::Analysis;
use crate::dynamics::{Diffusion, Dynamics};
use crate::excitation::{DampeningManager, ExcitationManager};
use crate::graph::{
    build_chain, build_grid, build_random_graph, build_ring, build_scale_free, build_small_world,
};
use crate::node::NodeState;

#[derive(Clone, PartialEq)]
pub enum Topology {
    Random,
    Chain,
    Ring,
    Grid(usize, usize),
    SmallWorld,
    ScaleFree,
}

pub struct Simulation {
    pub graph: UnGraph<(), f32>,
    pub nodes: Vec<NodeState>,
    pub dynamics: Box<dyn Dynamics>,
    pub time: f32,
    pub paused: bool,
    pub dt: f32,
    pub topology: Topology,
    pub node_count: usize,
    pub edge_prob: f32,
    pub grid_rows: usize,
    pub grid_cols: usize,
    pub analysis: Analysis,
    pub excitation: ExcitationManager,
    pub dampening: DampeningManager,
    pub rng: SmallRng,
}

impl Simulation {
    pub fn new() -> Self {
        let mut rng = SmallRng::from_os_rng();
        let node_count = 8;
        let edge_prob = 0.4;
        let dt = 0.01;

        let graph = build_random_graph(node_count, edge_prob, &mut rng);
        let dynamics: Box<dyn Dynamics> = Box::new(Diffusion::new());
        let nodes = dynamics.init_nodes(graph.node_count(), &mut rng);
        let analysis = Analysis::new(graph.node_count());

        Self {
            graph,
            nodes,
            dynamics,
            time: 0.0,
            paused: false,
            dt,
            topology: Topology::Random,
            node_count,
            edge_prob,
            grid_rows: 4,
            grid_cols: 4,
            analysis,
            excitation: ExcitationManager::new(),
            dampening: DampeningManager::new(),
            rng,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.excitation.apply(&mut self.nodes, self.time, dt);
        self.dampening.apply(&mut self.nodes, self.time, dt);
        self.dynamics.step(&self.graph, &mut self.nodes, dt);
        self.time += dt;
        let display_vals: Vec<f32> = self
            .nodes
            .iter()
            .map(|n| self.dynamics.display_value(n))
            .collect();
        self.analysis.update(&display_vals, self.time);
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.dynamics.reset_nodes(&mut self.nodes);
        self.time = 0.0;
        self.analysis.reset(self.nodes.len());
    }

    pub fn restart(&mut self) {
        self.dynamics.reset_nodes(&mut self.nodes);
        self.time = 0.0;
        self.analysis.restart();
        self.excitation.clear();
        self.dampening.clear();
    }

    pub fn fire_excitation(&mut self, node_idx: usize) {
        self.excitation.fire(node_idx, self.time);
    }

    pub fn randomize(&mut self) {
        self.graph = build_random_graph(self.node_count, self.edge_prob, &mut self.rng);
        self.nodes = self.dynamics.init_nodes(self.graph.node_count(), &mut self.rng);
        self.time = 0.0;
        self.analysis.reset(self.nodes.len());
    }

    pub fn set_dynamics(&mut self, new_dynamics: Box<dyn Dynamics>) {
        self.dynamics = new_dynamics;
        self.nodes = self.dynamics.init_nodes(self.graph.node_count(), &mut self.rng);
        self.time = 0.0;
        self.analysis.reset(self.nodes.len());
        self.excitation.set_dynamics(self.dynamics.name());
        self.dampening.set_dynamics(self.dynamics.name());
    }

    pub fn rebuild_topology(&mut self) {
        self.graph = match &self.topology {
            Topology::Random => {
                build_random_graph(self.node_count, self.edge_prob, &mut self.rng)
            }
            Topology::Chain => build_chain(self.node_count),
            Topology::Ring => build_ring(self.node_count),
            Topology::Grid(r, c) => build_grid(*r, *c),
            Topology::SmallWorld => {
                build_small_world(self.node_count, 4, 0.3, &mut self.rng)
            }
            Topology::ScaleFree => build_scale_free(self.node_count, 2, &mut self.rng),
        };
        self.nodes = self.dynamics.init_nodes(self.graph.node_count(), &mut self.rng);
        self.time = 0.0;
        self.analysis.reset(self.nodes.len());
        self.excitation.clear();
    }
}
