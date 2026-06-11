//! Sparse-set component storage.
//!
//! Components live in a dense `Vec<T>` (cache-friendly to iterate) with a
//! parallel `Vec<EntityId>` recording which entity owns each dense slot. A
//! sparse array maps entity index → dense slot for O(1) insert/get/remove;
//! removal swap-pops so the dense arrays never have holes.
//!
//! Generation safety: the sparse slot stores the owning entity's id, so a
//! lookup with a stale handle (same index, older generation) misses.

use super::entity::EntityId;

const EMPTY: u32 = u32::MAX;

#[derive(Debug)]
pub struct SparseSet<T> {
    /// entity index → dense slot, EMPTY if absent.
    sparse: Vec<u32>,
    /// dense slot → owning entity (id incl. generation).
    entities: Vec<EntityId>,
    dense: Vec<T>,
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self {
            sparse: Vec::new(),
            entities: Vec::new(),
            dense: Vec::new(),
        }
    }
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn dense_index(&self, id: EntityId) -> Option<usize> {
        let slot = *self.sparse.get(id.index() as usize)?;
        if slot == EMPTY {
            return None;
        }
        // Generation check: the dense slot must be owned by this exact id.
        (self.entities[slot as usize] == id).then_some(slot as usize)
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.dense_index(id).is_some()
    }

    /// Insert or replace. Returns the previous value if the entity already
    /// had one. If the sparse slot is held by a stale generation of the same
    /// index, the stale component is evicted so it can't linger in dense
    /// storage and show up during iteration.
    pub fn insert(&mut self, id: EntityId, value: T) -> Option<T> {
        let index = id.index() as usize;
        if index < self.sparse.len() && self.sparse[index] != EMPTY {
            let slot = self.sparse[index] as usize;
            if self.entities[slot] == id {
                return Some(std::mem::replace(&mut self.dense[slot], value));
            }
            self.remove(self.entities[slot]);
        }
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, EMPTY);
        }
        self.sparse[index] = self.dense.len() as u32;
        self.entities.push(id);
        self.dense.push(value);
        None
    }

    /// Remove and return the entity's component, swap-popping the dense
    /// arrays to stay contiguous.
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        let slot = self.dense_index(id)?;
        self.sparse[id.index() as usize] = EMPTY;
        let last = self.dense.len() - 1;
        if slot != last {
            // Move the tail element into the vacated slot and repoint its
            // sparse entry.
            let moved = self.entities[last];
            self.entities.swap(slot, last);
            self.dense.swap(slot, last);
            self.sparse[moved.index() as usize] = slot as u32;
        }
        self.entities.pop();
        self.dense.pop()
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.dense_index(id).map(|i| &self.dense[i])
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self.dense_index(id).map(|i| &mut self.dense[i])
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn clear(&mut self) {
        self.sparse.clear();
        self.entities.clear();
        self.dense.clear();
    }

    /// Iterate (owner, component) over dense storage — contiguous memory,
    /// no per-entity indirection.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.entities.iter().copied().zip(self.dense.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.entities.iter().copied().zip(self.dense.iter_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::super::entity::EntityAllocator;
    use super::*;

    #[test]
    fn insert_get_remove_roundtrip() {
        let mut alloc = EntityAllocator::new();
        let mut set = SparseSet::new();
        let a = alloc.allocate();
        let b = alloc.allocate();
        assert_eq!(set.insert(a, 10), None);
        assert_eq!(set.insert(b, 20), None);
        assert_eq!(set.get(a), Some(&10));
        assert_eq!(set.insert(a, 11), Some(10));
        assert_eq!(set.remove(a), Some(11));
        assert_eq!(set.get(a), None);
        assert_eq!(set.get(b), Some(&20), "swap-remove must repoint survivor");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn stale_generation_misses() {
        let mut alloc = EntityAllocator::new();
        let mut set = SparseSet::new();
        let a = alloc.allocate();
        set.insert(a, 1);
        alloc.deallocate(a);
        let b = alloc.allocate(); // same index, new generation
        assert_eq!(set.get(b), None, "new entity must not see old component");
        assert!(set.contains(a), "old id still owns the slot until removed");
        set.insert(b, 2);
        assert_eq!(set.get(b), Some(&2));
        assert_eq!(set.get(a), None, "stale id evicted by new insert");
        assert_eq!(set.len(), 1, "eviction must not leave a zombie in dense");
    }

    #[test]
    fn dense_iteration_visits_all() {
        let mut alloc = EntityAllocator::new();
        let mut set = SparseSet::new();
        let ids: Vec<_> = (0..100).map(|i| {
            let id = alloc.allocate();
            set.insert(id, i);
            id
        }).collect();
        // Remove every third to force swap-pops.
        for id in ids.iter().step_by(3) {
            set.remove(*id);
        }
        let visited: Vec<_> = set.iter().map(|(_, &v)| v).collect();
        assert_eq!(visited.len(), set.len());
        assert!(visited.iter().all(|v| v % 3 != 0));
    }
}
