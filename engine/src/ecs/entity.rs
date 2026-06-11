//! Generational entity allocator.
//!
//! An [`EntityId`] is an index into per-component sparse arrays plus a
//! generation counter. Despawning bumps the slot's generation and recycles the
//! index through a free list, so a held id from a previous lifetime of the
//! slot can never alias the new occupant — `is_alive` and every component
//! lookup check the generation.

/// Handle to an entity. Copyable, cheap, and safe to hold across despawns
/// (lookups with a stale handle return `None` rather than aliasing a recycled
/// slot).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl EntityId {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn generation(self) -> u32 {
        self.generation
    }
}

/// Allocates entity slots, recycling indices through a free list.
#[derive(Debug, Default)]
pub struct EntityAllocator {
    /// Generation per slot. Even = the slot is live at this generation,
    /// odd would over-complicate things — instead liveness is tracked by
    /// `free` membership, encoded here as: a slot is live iff its index is
    /// not on the free list. We keep an explicit `alive` bitvec-as-Vec<bool>
    /// for O(1) checks.
    generations: Vec<u32>,
    alive: Vec<bool>,
    free: Vec<u32>,
    live_count: usize,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self) -> EntityId {
        self.live_count += 1;
        if let Some(index) = self.free.pop() {
            self.alive[index as usize] = true;
            EntityId {
                index,
                generation: self.generations[index as usize],
            }
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            self.alive.push(true);
            EntityId {
                index,
                generation: 0,
            }
        }
    }

    /// Free the slot. Returns false if the id was already stale.
    pub fn deallocate(&mut self, id: EntityId) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        self.alive[id.index as usize] = false;
        self.generations[id.index as usize] += 1;
        self.free.push(id.index);
        self.live_count -= 1;
        true
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.alive
            .get(id.index as usize)
            .copied()
            .unwrap_or(false)
            && self.generations[id.index as usize] == id.generation
    }

    pub fn len(&self) -> usize {
        self.live_count
    }

    pub fn is_empty(&self) -> bool {
        self.live_count == 0
    }

    /// Capacity of the sparse arrays (highest index ever allocated + 1).
    pub fn slot_count(&self) -> usize {
        self.generations.len()
    }

    pub fn clear(&mut self) {
        self.generations.clear();
        self.alive.clear();
        self.free.clear();
        self.live_count = 0;
    }

    /// Iterate live entity ids in slot order.
    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, &a)| a)
            .map(|(i, _)| EntityId {
                index: i as u32,
                generation: self.generations[i],
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_is_sequential_from_zero() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        let b = alloc.allocate();
        assert_eq!((a.index(), a.generation()), (0, 0));
        assert_eq!((b.index(), b.generation()), (1, 0));
        assert_eq!(alloc.len(), 2);
    }

    #[test]
    fn deallocate_recycles_index_with_new_generation() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        assert!(alloc.deallocate(a));
        assert!(!alloc.is_alive(a));
        let b = alloc.allocate();
        assert_eq!(b.index(), a.index());
        assert_eq!(b.generation(), a.generation() + 1);
        assert!(alloc.is_alive(b));
        assert!(!alloc.is_alive(a), "stale handle must stay dead");
    }

    #[test]
    fn double_deallocate_is_rejected() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.allocate();
        assert!(alloc.deallocate(a));
        assert!(!alloc.deallocate(a));
        assert_eq!(alloc.len(), 0);
    }
}
