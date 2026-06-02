use std::hash::Hash;

use super::GlobalAssetId;
use super::asset_id::{AssetHandle, AssetId};

pub trait Asset: Sized + 'static {
    type Key: Eq + Hash + Clone;

    fn key(&self) -> &Self::Key;

    fn dependencies(&self) -> Vec<GlobalAssetId> {
        Vec::new()
    }
}

#[derive(Debug)]
struct Slot<T> {
    generation: u32,
    version: u32,
    value: Option<T>,
}

pub struct AssetStorage<T>
where
    T: Asset,
{
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
}

impl<T> Default for AssetStorage<T>
where
    T: Asset,
{
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }
}

impl<T> AssetStorage<T>
where
    T: Asset,
{
    pub fn insert(&mut self, asset: T) -> AssetHandle<T> {
        self.allocate(asset)
    }

    fn allocate(&mut self, asset: T) -> AssetHandle<T> {
        // Reuse free slot
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];

            debug_assert!(slot.value.is_none());

            slot.value = Some(asset);
            slot.version = 0;

            return AssetHandle::new(AssetId {
                index,
                generation: slot.generation,
            });
        }

        // Create new slot
        let index = self.slots.len() as u32;

        self.slots.push(Slot {
            generation: 0,
            version: 0,
            value: Some(asset),
        });

        AssetHandle::new(AssetId {
            index,
            generation: 0,
        })
    }

    // Get immutable
    pub fn get(&self, handle: AssetHandle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.id().index as usize)?;

        // Invalid stale handle
        if slot.generation != handle.id().generation {
            return None;
        }

        slot.value.as_ref()
    }

    // Get mutable
    pub fn get_mut(&mut self, handle: AssetHandle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.id().index as usize)?;

        if slot.generation != handle.id().generation {
            return None;
        }

        // Asset content changed
        slot.version += 1;

        slot.value.as_mut()
    }

    // Remove
    pub fn remove(&mut self, handle: AssetHandle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.id().index as usize)?;

        // Stale handle
        if slot.generation != handle.id().generation {
            return None;
        }

        // Already empty
        if slot.value.is_none() {
            return None;
        }

        // Invalidate old handles
        slot.generation += 1;
        slot.version = 0;

        self.free_list.push(handle.id().index);

        let value = slot.value.take()?;

        Some(value)
    }

    pub fn contains(&self, id: AssetId) -> bool {
        let Some(slot) = self.slots.get(id.index as usize) else {
            return false;
        };

        slot.generation == id.generation && slot.value.is_some()
    }
}

impl<T: Asset> AssetStorage<T> {
    pub fn remove_by_id(&mut self, id: AssetId) {
        let Some(slot) = self.slots.get_mut(id.index as usize) else {
            return;
        };

        if slot.generation != id.generation {
            return;
        }

        if slot.value.is_none() {
            return;
        }

        slot.value.take();

        slot.generation += 1;
        slot.version = 0;

        self.free_list.push(id.index);
    }
}
