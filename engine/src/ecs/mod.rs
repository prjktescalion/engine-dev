//! Entity-Component System. Backed by `hecs` once the dep is enabled.

/// World holds entities, components, and registered systems. Placeholder
/// fields until `hecs::World` is wired up.
pub struct World {
    pub entity_count: usize,
}

impl World {
    pub fn new() -> Self {
        Self { entity_count: 0 }
    }

    /// Run all registered systems for one tick.
    pub fn run_systems(&mut self, _dt: f32) {
        // TODO: iterate registered systems
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
