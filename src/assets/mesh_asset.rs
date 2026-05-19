use std::cell::Cell;

use super::*;

pub struct MeshDesc {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
    pub bounds: BoundingBox,
}

impl MeshDesc {
    /// estimete size in bytes
    pub fn estimated_size(&self) -> usize {
        self.vertices.len() * size_of::<MeshVertexData>() + self.indices.len() * size_of::<u32>()
    }
}

pub struct SubMesh {
    pub index_range: std::ops::Range<u32>,
    #[allow(dead_code)]
    pub base_vertex: u32,
    pub material: MaterialId,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum MeshSource {
    File {
        path: PathBuf,
        submesh_index: usize, // submesh index nel file
    },
    #[allow(dead_code)]
    Generated { shape: Primitive, params: [u32; 4] },
}

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Clone)]
pub enum Primitive {
    Cube,
    Quad,
    Sphere,
    Cylinder,
    Grid,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct MeshKey {
    pub source: MeshSource,
}

pub struct MeshAsset {
    pub desc: MeshDesc,
    ref_count: Cell<u32>,
}

#[derive(Default)]
pub struct MeshAssets {
    storage: SlotMap<MeshId, MeshAsset>,
    lookup: HashMap<MeshKey, MeshId>,
    stats: ResourceStats,
}

impl HasStats for MeshAssets {
    fn get_stats(&self) -> ResourceStats {
        self.stats.clone()
    }
}

impl MeshAssets {
    pub fn get_or_create(&mut self, key: MeshKey, desc_fn: impl FnOnce() -> MeshDesc) -> MeshId {
        if let Some(id) = self.lookup.get(&key) {
            let asset = self.storage.get(*id).unwrap();
            asset.ref_count.set(asset.ref_count.get() + 1);
            self.stats.add_shared();
            return *id;
        }

        let desc = desc_fn();
        let size = desc.estimated_size();
        let id = self.storage.insert(MeshAsset {
            desc,
            ref_count: Cell::new(1),
        });
        self.lookup.insert(key, id);
        self.stats.add(size);
        id
    }

    pub fn remove(&mut self, id: MeshId) {
        if let Some(asset) = self.storage.get(id) {
            let count = asset.ref_count.get();

            if count > 1 {
                asset.ref_count.set(count - 1);
                self.stats.remove_sahred();
            } else {
                let asset = self.storage.remove(id).unwrap();
                self.stats.remove(asset.desc.estimated_size());
                self.lookup.retain(|_key, &mut id| id != id);
            }
        }
    }

    pub fn contains_key(&self, id: MeshId) -> bool {
        self.storage.contains_key(id)
    }

    pub fn get(&self, id: MeshId) -> Option<&MeshDesc> {
        self.storage.get(id).map(|m| &m.desc)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MeshId, &MeshAsset)> {
        self.storage.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_mesh_same_id() {
        let mut meshes = MeshAssets::default();

        let source = MeshSource::File {
            path: "mesh.gltf".into(),
            submesh_index: 0,
        };
        let key = MeshKey {
            source: source.clone(),
        };
        let desc = MeshDesc {
            vertices: vec![MeshVertexData {
                ..Default::default()
            }],
            indices: Vec::new(),
            submeshes: Vec::new(),
            bounds: BoundingBox::new_empty(),
        };

        let key2 = MeshKey { source };
        let desc2 = MeshDesc {
            vertices: vec![MeshVertexData {
                ..Default::default()
            }],
            indices: Vec::new(),
            submeshes: Vec::new(),
            bounds: BoundingBox::new_empty(),
        };

        let a = meshes.get_or_create(key, || desc);
        let b = meshes.get_or_create(key2, || desc2);
        assert_eq!(a, b);
    }

    #[test]
    fn should_not_remove_shared_from_asset() {
        let mut meshes = MeshAssets::default();

        let source = MeshSource::File {
            path: "mesh.gltf".into(),
            submesh_index: 0,
        };
        let key = MeshKey { source };
        let desc = MeshDesc {
            vertices: Vec::new(),
            indices: Vec::new(),
            submeshes: Vec::new(),
            bounds: BoundingBox::new_empty(),
        };
        let desc2 = MeshDesc {
            vertices: vec![MeshVertexData {
                ..Default::default()
            }],
            indices: Vec::new(),
            submeshes: Vec::new(),
            bounds: BoundingBox::new_empty(),
        };

        let _id = meshes.get_or_create(key.clone(), || desc);
        let id = meshes.get_or_create(key, || desc2);

        meshes.remove(id);
        assert!(meshes.get(id).is_some());

        // now will remove ..
        meshes.remove(id);
        assert!(meshes.get(id).is_none());
    }

    #[test]
    fn should_have_stats() {
        fn assert_impl<T: HasStats>() {}
        assert_impl::<MeshAssets>();
    }

    #[test]
    fn should_increment_stats_on_add() {
        let mut meshes = MeshAssets::default();
        let initial_stats = meshes.get_stats();

        let source = MeshSource::File {
            path: "mesh.gltf".into(),
            submesh_index: 0,
        };
        let key = MeshKey { source };
        let desc = MeshDesc {
            vertices: vec![MeshVertexData {
                ..Default::default()
            }],
            indices: Vec::new(),
            submeshes: Vec::new(),
            bounds: BoundingBox::new_empty(),
        };
        let _ = meshes.get_or_create(key, || desc);

        let new_stats = meshes.get_stats();

        assert!(new_stats.count > initial_stats.count);
        assert!(new_stats.estimated_bytes > initial_stats.estimated_bytes);
    }

    #[test]
    fn should_decrements_stats_on_remove() {
        let mut meshes = MeshAssets::default();
        let initial_stats = meshes.get_stats();

        let source = MeshSource::File {
            path: "mesh.gltf".into(),
            submesh_index: 0,
        };
        let key = MeshKey { source };
        let desc = MeshDesc {
            vertices: vec![MeshVertexData {
                ..Default::default()
            }],
            indices: Vec::new(),
            submeshes: Vec::new(),
            bounds: BoundingBox::new_empty(),
        };
        let id = meshes.get_or_create(key, || desc);

        meshes.remove(id);
        let new_stats = meshes.get_stats();

        assert_eq!(new_stats.count, initial_stats.count);
        assert_eq!(new_stats.estimated_bytes, initial_stats.estimated_bytes);
    }
}
