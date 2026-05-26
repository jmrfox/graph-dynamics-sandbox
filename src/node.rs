#[derive(Clone, Debug)]
pub struct NodeState {
    pub slots: [f32; 4],
}

impl NodeState {
    pub fn zero() -> Self {
        Self { slots: [0.0; 4] }
    }

    pub fn s(&self, i: usize) -> f32 {
        self.slots[i]
    }

    pub fn s_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.slots[i]
    }
}
