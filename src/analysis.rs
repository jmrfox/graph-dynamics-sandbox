use std::collections::VecDeque;

const HISTORY_CAP: usize = 512;

pub struct Analysis {
    pub energy_history: VecDeque<(f32, f32)>,
    pub node_histories: Vec<VecDeque<f32>>,
    pub selected_nodes: Vec<bool>,
    pub order_history: VecDeque<(f32, f32)>,
}

impl Analysis {
    pub fn new(node_count: usize) -> Self {
        let mut selected_nodes = vec![false; node_count];
        if node_count > 0 {
            selected_nodes[0] = true;
        }
        Self {
            energy_history: VecDeque::with_capacity(HISTORY_CAP),
            node_histories: (0..node_count)
                .map(|_| VecDeque::with_capacity(HISTORY_CAP))
                .collect(),
            selected_nodes,
            order_history: VecDeque::with_capacity(HISTORY_CAP),
        }
    }

    pub fn reset(&mut self, node_count: usize) {
        self.energy_history.clear();
        self.order_history.clear();
        self.node_histories = (0..node_count)
            .map(|_| VecDeque::with_capacity(HISTORY_CAP))
            .collect();
        self.selected_nodes = vec![false; node_count];
        if node_count > 0 {
            self.selected_nodes[0] = true;
        }
    }

    pub fn restart(&mut self) {
        self.energy_history.clear();
        self.order_history.clear();
        for h in self.node_histories.iter_mut() {
            h.clear();
        }
    }

    pub fn update(&mut self, display_vals: &[f32], time: f32) {
        let e: f32 = display_vals.iter().map(|v| v * v).sum();
        push_capped(&mut self.energy_history, (time, e));

        let n = display_vals.len();
        if self.node_histories.len() != n {
            self.reset(n);
            return;
        }
        for (i, &v) in display_vals.iter().enumerate() {
            push_capped(&mut self.node_histories[i], v);
        }
    }

    pub fn push_order(&mut self, time: f32, r: f32) {
        push_capped(&mut self.order_history, (time, r));
    }
}

fn push_capped<T>(deque: &mut VecDeque<T>, val: T) {
    if deque.len() >= HISTORY_CAP {
        deque.pop_front();
    }
    deque.push_back(val);
}
