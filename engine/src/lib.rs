//! NeuDel-II Game Engine — runtime crate.
//!
//! Wireframe-level MVP: an [`Engine`] owns a [`World`] of entities with
//! transforms + optional velocity, plus subsystem stubs that will grow into
//! real wgpu/rapier2d/rodio backends. The studio drives a live simulation by
//! calling [`Engine::tick`] on a timer.

pub mod audio;
pub mod ecs;
pub mod physics;
pub mod renderer;
pub mod scripting;

use audio::Audio;
use ecs::{EntityId, World};
use physics::PhysicsWorld;
use renderer::Renderer;
use scripting::ScriptHost;

/// Top-level engine handle. Construct one per game/preview session.
pub struct Engine {
    pub world: World,
    pub renderer: Renderer,
    pub physics: PhysicsWorld,
    pub audio: Audio,
    pub scripts: ScriptHost,
    pub elapsed: f32,
    pub tick_count: u64,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            renderer: Renderer::new(),
            physics: PhysicsWorld::new(),
            audio: Audio::new(),
            scripts: ScriptHost::new(),
            elapsed: 0.0,
            tick_count: 0,
        }
    }

    /// Reset world + counters. Subsystem state is preserved.
    pub fn reset(&mut self) {
        self.world.clear();
        self.elapsed = 0.0;
        self.tick_count = 0;
    }

    /// Advance the simulation by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        self.scripts.update(dt);
        self.physics.step(dt);
        self.world.run_systems(dt);
        self.elapsed += dt;
        self.tick_count += 1;
    }

    /// Render the current world. Stub until wgpu lands; for now the studio
    /// reads `world.entities` directly to build its preview.
    pub fn render(&mut self) {
        self.renderer.draw(&self.world);
    }

    pub fn entity_count(&self) -> usize {
        self.world.len()
    }

    pub fn spawn(&mut self, name: impl Into<String>) -> EntityId {
        self.world.spawn(name)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
