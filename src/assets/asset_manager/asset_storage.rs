use std::collections::HashMap;
use std::hash::Hash;

use super::asset_id::*;
use crate::ResourceId;

pub trait Asset: Sized + 'static {
    type Key: Eq + Hash + Clone;

    fn key(&self) -> &Self::Key;

    fn dependencies(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    fn estimated_size(&self) -> usize {
        0
    }
}

#[derive(Debug)]
struct Slot<T> {
    version: u32,
    value: T,
}

pub struct AssetStorage<T: Asset> {
    slots: HashMap<ResourceId, Slot<T>>,
}

impl<T: Asset> Default for AssetStorage<T> {
    fn default() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }
}

impl<T: Asset> AssetStorage<T> {
    pub fn insert(&mut self, id: ResourceId, asset: T) -> AssetHandle<T> {
        debug_assert!(!self.slots.contains_key(&id));

        self.slots.insert(
            id,
            Slot {
                version: 0,
                value: asset,
            },
        );

        AssetHandle::new(id)
    }

    pub fn get_mut(&mut self, handle: AssetHandle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(&handle.id())?;
        slot.version = slot.version.wrapping_add(1);
        Some(&mut slot.value)
    }

    pub fn remove_by_id(&mut self, id: ResourceId) -> usize {
        self.slots
            .remove(&id)
            .map_or(0, |slot| slot.value.estimated_size())
    }

    pub fn iter(&self) -> impl Iterator<Item = (ResourceId, &T)> {
        self.slots
            .iter()
            .map(|(id, slot)| (*id, &slot.value))
    }

    pub fn get_by_id(&self, id: ResourceId) -> Option<&T> {
        self.slots.get(&id).map(|slot| &slot.value)
    }
}
