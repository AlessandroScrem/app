#![allow(unused)]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::assets::asset_id::{AssetHandle, AssetId};
use crate::assets::asset_manager2::AssetManager;
use crate::assets::asset_manager2::AssetType;


#[derive(Clone)]
pub enum StorageOp<T: Asset> {
    Created(AssetHandle<T>),
    Existing(AssetHandle<T>),
    Modified(AssetHandle<T>),
    Removed(AssetHandle<T>),
}

pub trait Asset: Sized + 'static {
    type Key: Eq + Hash + Clone;
    const TYPE: AssetType;

    fn key(&self) -> &Self::Key;
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

    lookup: HashMap<T::Key, AssetHandle<T>>,
}

impl<T> Default for AssetStorage<T>
where
    T: Asset,
{
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),

            // deduplication lookup
            lookup: HashMap::new(),
        }
    }
}

impl<T> AssetStorage<T>
where
    T: Asset,
{
    pub fn insert(&mut self, asset: T) -> StorageOp<T> {
        let key = asset.key().clone();

        if let Some(handle) = self.lookup.get(&key) {
            return StorageOp::Existing(*handle);
        }

        let handle = self.allocate(asset);
        self.lookup.insert(key, handle);

        StorageOp::Created(handle)
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
    pub fn remove(&mut self, handle: AssetHandle<T>) -> Option<StorageOp<T>> {
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

        let asset = slot.value.take()?;

        self.free_list.push(handle.id().index);

        // remove from dedup llookup

        let key = asset.key();
        self.lookup.remove(&key);

        Some(StorageOp::Removed(handle))
    }

    // Version
    pub fn version(&self, handle: AssetHandle<T>) -> Option<u32> {
        let slot = self.slots.get(handle.id().index as usize)?;

        if slot.generation != handle.id().generation {
            return None;
        }

        Some(slot.version)
    }

    // Exists
    pub fn contains(&self, handle: AssetHandle<T>) -> bool {
        self.get(handle).is_some()
    }

    // Exists
    fn contains_key(&self, key: &T::Key) -> bool {
        self.lookup.contains_key(&key)
    }

    // Number of slots
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example assets
    #[derive(Debug)]
    struct Mesh {
        name: String,
    }

    impl Asset for Mesh {
        type Key = String;
        const TYPE: AssetType = AssetType::Mesh;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }

    #[derive(Debug)]
    struct Texture {
        name: String,
        width: u32,
        height: u32,
    }

    impl Asset for Texture {
        type Key = String;
        const TYPE: AssetType = AssetType::Texture;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }

    #[test]
    fn insert_and_get() {
        let mut storage = AssetStorage::<Mesh>::default();

        let handle = match storage.insert(Mesh {
            name: "Cube".into(),
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        let mesh = storage.get(handle).unwrap();

        assert_eq!(mesh.name, "Cube");
    }

    #[test]
    fn deduplicate_works() {
        let mut storage = AssetStorage::<Mesh>::default();

        let h1 = match storage.insert(Mesh {
            name: "Cube".into(),
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        let h2 = match storage.insert(Mesh {
            name: "Cube".into(),
        }) {
            StorageOp::Existing(h) => h,
            _ => panic!("exprected existing"),
        };
        assert_eq!(h1.id().index, h2.id().index);
        assert_eq!(storage.capacity(), 1);
    }

    #[test]
    fn remove_will_update_lookup() {
        let mut storage = AssetStorage::<Mesh>::default();

        let handle = match storage.insert(Mesh {
            name: "Cube".into(),
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        let removed = storage.remove(handle);

        assert!(matches!(removed, Some(StorageOp::Removed(_))));
        assert!(!storage.contains_key(&"Cube".to_string()));
    }

    #[test]
    fn remove_invalidates_old_handle() {
        let mut storage = AssetStorage::<Mesh>::default();

        let handle_a = match storage.insert(Mesh {
            name: "MeshA".into(),
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        storage.remove(handle_a);

        // Old handle invalid
        assert!(storage.get(handle_a).is_none());

        // Slot reused
        let handle_b = match storage.insert(Mesh {
            name: "MeshB".into(),
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        // Same slot index
        assert_eq!(handle_a.id().index, handle_b.id().index);

        // Different generation
        assert_ne!(handle_a.id().generation, handle_b.id().generation);

        // Old handle still invalid
        assert!(storage.get(handle_a).is_none());

        // New handle valid
        assert_eq!(storage.get(handle_b).unwrap().name, "MeshB");
    }

    #[test]
    fn version_changes_on_mutation() {
        let mut storage = AssetStorage::<Mesh>::default();

        let handle = match storage.insert(Mesh {
            name: "Before".into(),
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        assert_eq!(storage.version(handle), Some(0));

        {
            let mesh = storage.get_mut(handle).unwrap();
            mesh.name = "After".into();
        }

        assert_eq!(storage.version(handle), Some(1));
    }

    #[test]
    fn contains_works() {
        let mut storage = AssetStorage::<Texture>::default();

        let handle = match storage.insert(Texture {
            name: "Texture".into(),
            width: 512,
            height: 512,
        }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        assert!(storage.contains(handle));

        storage.remove(handle);

        assert!(!storage.contains(handle));
    }

    #[test]
    fn stale_handle_cannot_remove_new_asset() {
        let mut storage = AssetStorage::<Mesh>::default();

        let old_handle = match storage.insert(Mesh { name: "Old".into() }) {
            StorageOp::Created(h) => h,
            _ => panic!(),
        };
        
        storage.remove(old_handle);
        
        let new_handle = match storage.insert(Mesh { name: "New".into() }){
            StorageOp::Created(h) => h,
            _ => panic!(),
        };

        // Stale handle cannot delete new asset
        assert!(storage.remove(old_handle).is_none());

        // New asset still exists
        assert!(storage.contains(new_handle));
    }
}
