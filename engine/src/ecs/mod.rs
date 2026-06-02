//! Entity-Component System.
//!
//! Wireframe-level MVP: a single `World` struct holding a `Vec<Entity>` where
//! each entity carries an id, a transform, and optional velocity / sprite /
//! tag components. The intent is to swap this for `hecs` once the system trait
//! shape stabilizes — for now this is the smallest thing that lets the studio
//! drive a real simulation.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

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

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub transform: Transform,
    pub velocity: Option<Velocity>,
    pub sprite: Option<Sprite>,
}

pub struct World {
    pub entities: Vec<Entity>,
    next_id: AtomicU64,
    /// World-space bounds used by the integrator to bounce moving entities so
    /// the wireframe demo stays on-screen. The studio sets this from canvas
    /// size; defaults to a reasonable viewport.
    pub bounds: (f32, f32, f32, f32),
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: AtomicU64::new(1),
            bounds: (0.0, 0.0, 1024.0, 600.0),
        }
    }

    pub fn spawn(&mut self, name: impl Into<String>) -> EntityId {
        let id = EntityId(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.entities.push(Entity {
            id,
            name: name.into(),
            transform: Transform::default(),
            velocity: None,
            sprite: None,
        });
        id
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn set_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.bounds = (min_x, min_y, max_x, max_y);
    }

    /// Integrate velocity into transform for one tick. Entities outside the
    /// world bounds bounce — convenient for the demo, harmless when nothing
    /// has velocity.
    pub fn run_systems(&mut self, dt: f32) {
        let (min_x, min_y, max_x, max_y) = self.bounds;
        for e in &mut self.entities {
            let Some(v) = e.velocity.as_mut() else { continue };
            e.transform.x += v.vx * dt;
            e.transform.y += v.vy * dt;
            e.transform.rotation += v.vrot * dt;
            if e.transform.x < min_x {
                e.transform.x = min_x;
                v.vx = v.vx.abs();
            } else if e.transform.x > max_x {
                e.transform.x = max_x;
                v.vx = -v.vx.abs();
            }
            if e.transform.y < min_y {
                e.transform.y = min_y;
                v.vy = v.vy.abs();
            } else if e.transform.y > max_y {
                e.transform.y = max_y;
                v.vy = -v.vy.abs();
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
