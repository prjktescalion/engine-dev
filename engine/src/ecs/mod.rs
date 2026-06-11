//! Entity-Component System — from-scratch optimized core.
//!
//! Replaces the wireframe `Vec<Entity>` world (and the planned hecs/Bevy
//! dependency) with a sparse-set ECS in the EnTT style:
//!
//! - [`entity`]: generational ids + free-list allocator — O(1) spawn/despawn,
//!   stale handles can never alias a recycled slot.
//! - [`sparse_set`]: per-component dense storage — O(1) lookup, contiguous
//!   iteration, swap-pop removal.
//! - [`world`]: fixed roster of typed stores plus the movement system, which
//!   walks only entities that actually have a velocity.
//!
//! Compared to the old core: component lookup was O(n) (`Vec::iter().find`),
//! making the studio's per-frame write-back O(n²); both are now O(n) total.
//! `cargo run --release -p engine --example ecs_bench` has the numbers.

pub mod entity;
pub mod sparse_set;
pub mod world;

pub use entity::EntityId;
pub use sparse_set::SparseSet;
pub use world::World;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
    pub vrot: f32,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub asset_path: String,
}
