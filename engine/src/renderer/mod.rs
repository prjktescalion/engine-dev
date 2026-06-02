//! 2D/3D renderer. Backed by `wgpu` + `winit` once the deps are enabled.

use crate::ecs::World;

pub struct Renderer {
    pub clear_color: [f32; 4],
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            clear_color: [0.04, 0.05, 0.08, 1.0],
        }
    }

    pub fn draw(&mut self, _world: &World) {
        // TODO: build draw list from world, submit to wgpu queue
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
