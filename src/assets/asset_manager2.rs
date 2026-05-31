use crate::assets::asset_id::{AssetHandle, AssetId};
use crate::assets::asset_storage::{Asset, AssetStorage, StorageOp};

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

pub trait AssetAccess: Asset {
    fn storage(manager: &mut AssetManager) -> &mut AssetStorage<Self>;

    fn storage_ref(manager: &AssetManager) -> &AssetStorage<Self>;
}

#[derive(Default)]
pub struct AssetManager {
    meshes: AssetStorage<MeshAsset>,
    materials: AssetStorage<MaterialAsset>,
    textures: AssetStorage<TextureAsset>,

    events: Vec<AssetEvent>,
}

impl AssetManager {
    pub fn add<T: AssetAccess>(&mut self, asset: T) -> AssetHandle<T> {
        let op = T::storage(self).insert(asset);

        match self.push_event(op) {
            StorageOp::Created(h) | StorageOp::Existing(h) => h,
            _ => unreachable!(),
        }
    }

    pub fn modify<T, F>(&mut self, handle: AssetHandle<T>, f: F)
    where
        T: AssetAccess,
        F: FnOnce(&mut T),
    {
        if let Some(asset) = T::storage(self).get_mut(handle) {
            f(asset);

            self.events.push(AssetEvent {
                ty: T::TYPE,
                id: handle.id(),
                kind: AssetEventKind::Modified,
            });
        }
    }

    pub fn get<T: AssetAccess>(&self, handle: AssetHandle<T>) -> Option<&T> {
        T::storage_ref(self).get(handle)
    }

    pub fn remove<T: AssetAccess>(&mut self, handle: AssetHandle<T>) {
        if let Some(op) = T::storage(self).remove(handle) {
            self.push_event(op);
        }
    }

    fn push_event<T: Asset>(&mut self, op: StorageOp<T>) -> StorageOp<T> {
        if let Some(event) = op.to_asset_event() {
            self.events.push(event);
        }

        op
    }
}

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

impl AssetAccess for MeshAsset {
    fn storage(manager: &mut AssetManager) -> &mut AssetStorage<Self> {
        &mut manager.meshes
    }

    fn storage_ref(manager: &AssetManager) -> &AssetStorage<Self> {
        &manager.meshes
    }
}

impl AssetAccess for MaterialAsset {
    fn storage(manager: &mut AssetManager) -> &mut AssetStorage<Self> {
        &mut manager.materials
    }

    fn storage_ref(manager: &AssetManager) -> &AssetStorage<Self> {
        &manager.materials
    }
}

impl AssetAccess for TextureAsset {
    fn storage(manager: &mut AssetManager) -> &mut AssetStorage<Self> {
        &mut manager.textures
    }

    fn storage_ref(manager: &AssetManager) -> &AssetStorage<Self> {
        &manager.textures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------
    // Helpers
    // ----------------------------

    fn extract_created<T: AssetAccess>(op: StorageOp<T>) -> AssetHandle<T> {
        match op {
            StorageOp::Created(h) => h,
            _ => panic!("expected Created"),
        }
    }

    fn extract_existing<T: AssetAccess>(op: StorageOp<T>) -> AssetHandle<T> {
        match op {
            StorageOp::Existing(h) => h,
            _ => panic!("expected Existing"),
        }
    }

    // ----------------------------
    // Insert + get
    // ----------------------------

    #[test]
    fn insert_and_get_mesh() {
        let mut manager = AssetManager::default();

        let handle = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        let mesh = manager.get(handle).unwrap();
        assert_eq!(mesh.name, "Cube");

        // event created
        assert_eq!(manager.events.len(), 1);
    }

    // ----------------------------
    // Remove event + invalidation
    // ----------------------------

    #[test]
    fn remove_generates_event_and_invalidates_handle() {
        let mut manager = AssetManager::default();

        let handle = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        manager.remove(handle);

        // handle invalid
        assert!(manager.get(handle).is_none());

        // evento Removed presente
        assert!(
            manager
                .events
                .iter()
                .any(|e| matches!(e.kind, AssetEventKind::Removed))
        );
    }

    #[test]
    fn get_mut_changes_version_and_emits_event() {
        let mut manager = AssetManager::default();

        let handle = manager.add(MeshAsset {
            name: "Before".into(),
        });

        manager.modify(handle, |mesh| {
            mesh.name = "After".into();
        });

        let mesh = manager.get(handle).unwrap();
        assert_eq!(mesh.name, "After");

        //verifica evento Modified
        assert!(
            manager
                .events
                .iter()
                .any(|e| matches!(e.kind, AssetEventKind::Modified))
        );
    }

    // ----------------------------
    // Multi-type correctness
    // ----------------------------

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

        assert_eq!(manager.events.len(), 3);
    }

    // ----------------------------
    // Existing path (dedup)
    // ----------------------------

    #[test]
    fn existing_does_not_create_second_event() {
        let mut manager = AssetManager::default();

        let _ = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        let _ = manager.add(MeshAsset {
            name: "Cube".into(),
        });

        // solo 1 evento Created
        assert_eq!(manager.events.len(), 1);
    }
}
