use crate::assets::asset_id::{AssetHandle, AssetId};
use crate::assets::asset_storage::{Asset, AssetStorage, StorageOp};

use std::any::{Any, TypeId};
use std::collections::HashMap;

trait ErasedStorage {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct TypedStorage<T: Asset> {
    inner: AssetStorage<T>,
}

impl<T: Asset> ErasedStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct AssetManager {
    storages: HashMap<TypeId, Box<dyn ErasedStorage>>,
    events: Vec<AssetEvent>,
}

impl AssetManager {
    fn storage<T: Asset>(&self) -> &AssetStorage<T> {
        let id = TypeId::of::<T>();

        let storage = self.storages.get(&id).unwrap();

        &storage
            .as_any()
            .downcast_ref::<TypedStorage<T>>()
            .unwrap()
            .inner
    }

    fn storage_mut<T: Asset>(&mut self) -> &mut AssetStorage<T> {
        let id = TypeId::of::<T>();

        self.storages.entry(id).or_insert_with(|| {
            Box::new(TypedStorage::<T> {
                inner: AssetStorage::default(),
            })
        });

        let storage = self.storages.get_mut(&id).unwrap();

        &mut storage
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .unwrap()
            .inner
    }
}

#[derive(Debug)]
struct AssetEvent {
    pub ty: AssetType,
    pub id: AssetId,
    pub kind: AssetEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Mesh,
    Material,
    Texture,
}

#[derive(Debug, PartialEq)]
enum AssetEventKind {
    Created,
    Removed,
    Modified,
}

impl<T: Asset> StorageOp<T> {
    fn to_asset_event(&self) -> Option<AssetEvent> {
        match self {
            StorageOp::Created(handle) => Some(AssetEvent {
                ty: T::TYPE,
                id: handle.id(),
                kind: AssetEventKind::Created,
            }),

            StorageOp::Removed(handle) => Some(AssetEvent {
                ty: T::TYPE,
                id: handle.id(),
                kind: AssetEventKind::Removed,
            }),

            StorageOp::Existing(_) => None,

            StorageOp::Modified(handle) => Some(AssetEvent {
                ty: T::TYPE,
                id: handle.id(),
                kind: AssetEventKind::Modified,
            }),
        }
    }
}

impl AssetManager {
    pub fn add<T: Asset>(&mut self, asset: T) -> AssetHandle<T> {
        let op = self.storage_mut::<T>().insert(asset);

        self.push_event(op).into_handle()
    }

    pub fn get<T: Asset>(&self, handle: AssetHandle<T>) -> Option<&T> {
        self.storage::<T>().get(handle)
    }

    pub fn modify<T: Asset>(&mut self, handle: AssetHandle<T>, f: impl FnOnce(&mut T)) {
        if let Some(asset) = self.storage_mut::<T>().get_mut(handle) {
            f(asset);

            self.events.push(AssetEvent {
                ty: T::TYPE,
                id: handle.id(),
                kind: AssetEventKind::Modified,
            });
        }
    }

    pub fn remove<T: Asset>(&mut self, handle: AssetHandle<T>) {
        if let Some(op) = self.storage_mut::<T>().remove(handle) {
            self.push_event(op);
        }
    }

    fn push_event<T: Asset>(&mut self, op: StorageOp<T>) -> StorageOp<T> {
        if let Some(event) = op.to_asset_event() {
            self.events.push(event);
        }

        op
    }

    #[cfg(test)]
    fn event_count(&self) -> usize {
        self.events.len()
    }

    #[cfg(test)]
    fn has_event_kind(&self, kind: AssetEventKind) -> bool {
        self.events.iter().any(|e| e.kind == kind)
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            storages: HashMap::new(),
            events: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MeshAsset {
        name: String,
    }
    struct MaterialAsset {
        name: String,
    }
    struct TextureAsset {
        name: String,
    }

    impl Asset for MeshAsset {
        type Key = String;
        const TYPE: AssetType = AssetType::Mesh;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }
    impl Asset for MaterialAsset {
        type Key = String;
        const TYPE: AssetType = AssetType::Material;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }
    impl Asset for TextureAsset {
        type Key = String;
        const TYPE: AssetType = AssetType::Texture;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }

    #[test]
    fn insert_and_get_mesh() {
        let mut manager = AssetManager::default();

        let handle = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        let mesh = manager.get(handle).unwrap();
        assert_eq!(mesh.name, "Cube");

        assert_eq!(manager.event_count(), 1);
    }

    #[test]
    fn remove_generates_event_and_invalidates_handle() {
        let mut manager = AssetManager::default();

        let handle = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        manager.remove(handle);

        assert!(manager.get(handle).is_none());

        assert!(manager.has_event_kind(AssetEventKind::Removed));
    }

    #[test]
    fn modify_emits_event() {
        let mut manager = AssetManager::default();

        let handle = manager.add(MeshAsset {
            name: "Before".into(),
        });

        manager.modify(handle, |mesh| {
            mesh.name = "After".into();
        });

        let mesh = manager.get(handle).unwrap();
        assert_eq!(mesh.name, "After");

        assert!(manager.has_event_kind(AssetEventKind::Modified));
    }

    #[test]
    fn multiple_asset_types_work() {
        let mut manager = AssetManager::default();

        let mesh = manager.add(MeshAsset {
            name: "Mesh".into(),
        });
        let mat = manager.add(MaterialAsset { name: "Mat".into() });
        let tex = manager.add(TextureAsset { name: "Tex".into() });

        assert_eq!(manager.get(mesh).unwrap().name, "Mesh");
        assert_eq!(manager.get(mat).unwrap().name, "Mat");
        assert_eq!(manager.get(tex).unwrap().name, "Tex");

        assert_eq!(manager.event_count(), 3);
    }

    #[test]
    fn existing_does_not_create_second_event() {
        let mut manager = AssetManager::default();

        let h1 = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        let h2 = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        assert_eq!(h1.id().index, h2.id().index);

        // solo Created una volta
        assert_eq!(manager.event_count(), 1);
    }
}
