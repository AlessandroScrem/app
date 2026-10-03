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
    value: Option<T>,
}

pub struct AssetStorage<T>
where
    T: Asset,
{
    slots: Vec<Slot<T>>,
    free_list: Vec<usize>,
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

impl<T: Asset> AssetStorage<T> {
    pub fn insert(&mut self, id: ResourceId, asset: T) -> AssetHandle<T> {
        let index = id.raw();

        if index >= self.slots.len() {
            self.slots.resize_with(index + 1, || Slot {
                version: 0,
                value: None,
            });
        }

        debug_assert!(self.slots[index].value.is_none());
        self.slots[index] = Slot {
            version: 0,
            value: Some(asset),
        };

        AssetHandle::new(id)
    }

    pub fn get_mut(&mut self, handle: AssetHandle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.id().raw())?;

        if slot.value.is_none() {
            return None;
        }

        slot.version = slot.version.wrapping_add(1);
        slot.value.as_mut()
    }

    pub fn remove_by_id(&mut self, id: ResourceId) -> usize {
        let Some(slot) = self.slots.get_mut(id.raw()) else {
            return 0;
        };

        let Some(asset) = slot.value.take() else {
            return 0;
        };

        slot.version = 0;
        self.free_list.push(id.raw());

        asset.estimated_size()
    }

    pub fn iter(&self) -> impl Iterator<Item = (ResourceId, &T)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.value
                .as_ref()
                .map(|value| (ResourceId::from_raw(index), value))
        })
    }

    pub fn get_by_id(&self, id: ResourceId) -> Option<&T> {
        self.slots.get(id.raw())?.value.as_ref()
    }
}
