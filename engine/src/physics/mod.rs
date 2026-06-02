//! 2D physics. Backed by `rapier2d` once the dep is enabled.

pub struct PhysicsWorld {
    pub gravity: [f32; 2],
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            gravity: [0.0, -9.81],
        }
    }

    pub fn step(&mut self, _dt: f32) {
        // TODO: rapier2d::PhysicsPipeline::step
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
