//! World: entity allocator + one sparse set per component type.
//!
//! The component roster is fixed and concrete (no `TypeId` erasure): for an
//! engine this size, a named field per store is both simpler and faster than
//! a dynamic registry, and adding a component type is a three-line change.
//! Every live entity has a name and a transform; velocity and sprite are
//! optional.

use super::entity::{EntityAllocator, EntityId};
use super::sparse_set::SparseSet;
use super::{Sprite, Transform, Velocity};

pub struct World {
    allocator: EntityAllocator,
    names: SparseSet<String>,
    transforms: SparseSet<Transform>,
    velocities: SparseSet<Velocity>,
    sprites: SparseSet<Sprite>,
    /// World-space bounds used by the integrator to bounce moving entities so
    /// the wireframe demo stays on-screen. The studio sets this from canvas
    /// size; defaults to a reasonable viewport.
    pub bounds: (f32, f32, f32, f32),
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            names: SparseSet::new(),
            transforms: SparseSet::new(),
            velocities: SparseSet::new(),
            sprites: SparseSet::new(),
            bounds: (0.0, 0.0, 1024.0, 600.0),
        }
    }

    // ----- Lifecycle -------------------------------------------------------

    pub fn spawn(&mut self, name: impl Into<String>) -> EntityId {
        let id = self.allocator.allocate();
        self.names.insert(id, name.into());
        self.transforms.insert(id, Transform::default());
        id
    }

    /// Despawn the entity, removing all its components. Returns false if the
    /// id was already stale.
    pub fn despawn(&mut self, id: EntityId) -> bool {
        if !self.allocator.deallocate(id) {
            return false;
        }
        self.names.remove(id);
        self.transforms.remove(id);
        self.velocities.remove(id);
        self.sprites.remove(id);
        true
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.allocator.is_alive(id)
    }

    pub fn clear(&mut self) {
        self.allocator.clear();
        self.names.clear();
        self.transforms.clear();
        self.velocities.clear();
        self.sprites.clear();
    }

    pub fn len(&self) -> usize {
        self.allocator.len()
    }

    pub fn is_empty(&self) -> bool {
        self.allocator.is_empty()
    }

    /// Iterate live entity ids in slot order.
    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.allocator.iter()
    }

    // ----- Component access (all O(1)) -------------------------------------

    pub fn name(&self, id: EntityId) -> Option<&str> {
        self.names.get(id).map(String::as_str)
    }

    pub fn transform(&self, id: EntityId) -> Option<&Transform> {
        self.transforms.get(id)
    }

    pub fn transform_mut(&mut self, id: EntityId) -> Option<&mut Transform> {
        self.transforms.get_mut(id)
    }

    pub fn velocity(&self, id: EntityId) -> Option<&Velocity> {
        self.velocities.get(id)
    }

    pub fn velocity_mut(&mut self, id: EntityId) -> Option<&mut Velocity> {
        self.velocities.get_mut(id)
    }

    pub fn set_velocity(&mut self, id: EntityId, v: Velocity) {
        if self.is_alive(id) {
            self.velocities.insert(id, v);
        }
    }

    pub fn remove_velocity(&mut self, id: EntityId) -> Option<Velocity> {
        self.velocities.remove(id)
    }

    pub fn sprite(&self, id: EntityId) -> Option<&Sprite> {
        self.sprites.get(id)
    }

    pub fn set_sprite(&mut self, id: EntityId, s: Sprite) {
        if self.is_alive(id) {
            self.sprites.insert(id, s);
        }
    }

    pub fn remove_sprite(&mut self, id: EntityId) -> Option<Sprite> {
        self.sprites.remove(id)
    }

    pub fn set_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.bounds = (min_x, min_y, max_x, max_y);
    }

    // ----- Systems ----------------------------------------------------------

    /// Integrate velocity into transform for one tick. Walks the velocity
    /// store densely (only moving entities, contiguous memory) and probes the
    /// transform store O(1) per entity. Entities outside the world bounds
    /// bounce — convenient for the demo, harmless when nothing has velocity.
    pub fn run_systems(&mut self, dt: f32) {
        let (min_x, min_y, max_x, max_y) = self.bounds;
        let transforms = &mut self.transforms;
        for (id, v) in self.velocities.iter_mut() {
            let Some(t) = transforms.get_mut(id) else { continue };
            t.x += v.vx * dt;
            t.y += v.vy * dt;
            t.rotation += v.vrot * dt;
            if t.x < min_x {
                t.x = min_x;
                v.vx = v.vx.abs();
            } else if t.x > max_x {
                t.x = max_x;
                v.vx = -v.vx.abs();
            }
            if t.y < min_y {
                t.y = min_y;
                v.vy = v.vy.abs();
            } else if t.y > max_y {
                t.y = max_y;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_gives_name_and_default_transform() {
        let mut w = World::new();
        let e = w.spawn("player");
        assert_eq!(w.name(e), Some("player"));
        let t = w.transform(e).unwrap();
        assert_eq!((t.x, t.y, t.scale_x), (0.0, 0.0, 1.0));
        assert!(w.velocity(e).is_none());
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn despawn_removes_components_and_invalidates_handle() {
        let mut w = World::new();
        let e = w.spawn("enemy");
        w.set_velocity(e, Velocity { vx: 1.0, vy: 0.0, vrot: 0.0 });
        assert!(w.despawn(e));
        assert!(!w.is_alive(e));
        assert!(w.transform(e).is_none());
        assert!(w.velocity(e).is_none());
        assert!(!w.despawn(e), "double despawn rejected");
        // Recycled slot must start clean.
        let f = w.spawn("fresh");
        assert_eq!(f.index(), e.index());
        assert!(w.velocity(f).is_none());
        assert_eq!(w.name(f), Some("fresh"));
    }

    #[test]
    fn movement_system_integrates_and_bounces() {
        let mut w = World::new();
        w.set_bounds(0.0, 0.0, 100.0, 100.0);
        let e = w.spawn("ball");
        w.transform_mut(e).unwrap().x = 99.0;
        w.set_velocity(e, Velocity { vx: 10.0, vy: 0.0, vrot: 1.0 });
        w.run_systems(1.0);
        let t = w.transform(e).unwrap();
        assert_eq!(t.x, 100.0, "clamped to bound");
        assert_eq!(t.rotation, 1.0);
        assert!(w.velocity(e).unwrap().vx < 0.0, "bounced");
        // Static entities are untouched and don't cost iteration time.
        let s = w.spawn("rock");
        w.run_systems(1.0);
        let ts = w.transform(s).unwrap();
        assert_eq!((ts.x, ts.y), (0.0, 0.0));
    }

    #[test]
    fn set_velocity_on_dead_entity_is_noop() {
        let mut w = World::new();
        let e = w.spawn("ghost");
        w.despawn(e);
        w.set_velocity(e, Velocity { vx: 1.0, vy: 1.0, vrot: 0.0 });
        let f = w.spawn("live"); // recycles e's slot
        assert!(w.velocity(f).is_none());
    }
}
