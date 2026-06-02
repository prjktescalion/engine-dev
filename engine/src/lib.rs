//! NeuDel-II Game Engine — runtime crate.
//!
//! This crate is the engine *backend* the studio drives. Subsystems are
//! still stubs: real implementations land here once each module is wired
//! up (wgpu renderer, hecs ECS, rapier2d physics, rodio audio, scripting
//! bridges for Rust/Java/Python).
//!
//! Studio talks to the engine through the [`Engine`] struct and the per-
//! subsystem traits defined in each submodule.

pub mod audio;
pub mod ecs;
pub mod physics;
pub mod renderer;
pub mod scripting;

use audio::Audio;
use ecs::World;
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
}

impl Engine {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            renderer: Renderer::new(),
            physics: PhysicsWorld::new(),
            audio: Audio::new(),
            scripts: ScriptHost::new(),
        }
    }

    /// Advance the simulation by `dt` seconds. Stub; real fixed-timestep
    /// integration lives here once subsystems are real.
    pub fn tick(&mut self, dt: f32) {
        self.scripts.update(dt);
        self.physics.step(dt);
        self.world.run_systems(dt);
    }

    /// Render the current world. Stub.
    pub fn render(&mut self) {
        self.renderer.draw(&self.world);
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
