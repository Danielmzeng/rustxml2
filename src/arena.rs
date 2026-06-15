//! A simple generational arena. Stale `NodeId`s (after removal) are rejected.

/// Handle into an [`Arena`]. Cheap to copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: u32,
    generation: u32,
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// Generational arena storage.
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { slots: Vec::new(), free: Vec::new() }
    }

    pub fn insert(&mut self, value: T) -> NodeId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            NodeId { index, generation: slot.generation }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot { generation: 0, value: Some(value) });
            NodeId { index, generation: 0 }
        }
    }

    fn slot(&self, id: NodeId) -> Option<&Slot<T>> {
        self.slots
            .get(id.index as usize)
            .filter(|s| s.generation == id.generation && s.value.is_some())
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.slot(id).and_then(|s| s.value.as_ref())
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation || slot.value.is_none() {
            return None;
        }
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);
        slot.value.take()
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.slot(id).is_some()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_remove_with_generations() {
        let mut a: Arena<i32> = Arena::new();
        let id = a.insert(42);
        assert_eq!(a.get(id), Some(&42));

        a.remove(id);
        assert_eq!(a.get(id), None); // stale handle rejected

        let id2 = a.insert(7); // reuses the slot, new generation
        assert_eq!(a.get(id2), Some(&7));
        assert_eq!(a.get(id), None); // old handle still rejected
        assert_ne!(id, id2);
    }

    #[test]
    fn get_mut_mutates() {
        let mut a: Arena<i32> = Arena::new();
        let id = a.insert(1);
        *a.get_mut(id).unwrap() = 99;
        assert_eq!(a.get(id), Some(&99));
    }
}
