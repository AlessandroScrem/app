use super::*;

pub(crate) struct MeshDesc {
    pub(crate) vertices: Vec<MeshVertexData>,
    pub(crate) indices: Vec<u32>,
    pub(crate) submeshes: Vec<SubMesh>,
    pub(crate) bounds: BoundingBox,
}

pub(crate) struct SubMesh {
    pub(crate) index_range: std::ops::Range<u32>,
    pub(crate) base_vertex: u32,
    pub(crate) material: MaterialId,
}

#[derive(Hash, Eq, PartialEq)]
pub(crate) enum MeshSource {
    File {
        path: PathBuf,
        index: usize, // submesh index nel file
    },
    Generated {
        shape: Primitive,
        params: [u32; 4],
    },
}

#[derive(Hash, Eq, PartialEq)]
pub(crate) enum Primitive {
    Cube,
    Quad,
    Sphere,
    Cylinder,
    Grid,
}

#[derive(Hash, Eq, PartialEq)]
pub(crate) struct MeshKey {
    pub(crate) source: MeshSource,
}

#[derive(Default)]
pub(crate) struct MeshAssets {
    storage: SlotMap<MeshId, MeshDesc>,
    lookup: HashMap<MeshKey, MeshId>,
}

impl MeshAssets {
    pub(crate) fn get_or_create(&mut self, key: MeshKey, desc_fn: impl FnOnce() -> MeshDesc) -> MeshId {
        if let Some(id) = self.lookup.get(&key) {
            return *id;
        }

        let desc = desc_fn();
        let id = self.storage.insert(desc);
        self.lookup.insert(key, id);
        id
    }

    pub(crate) fn remove(&mut self, id: MeshId) {
        if self.storage.contains_key(id) {
            self.storage.remove(id);
            self.lookup.retain(|_key, &mut id| id != id);
        }
    }

    pub(crate) fn contains_key(&self, id: MeshId) ->bool {
        self.storage.contains_key(id)
    }

    pub(crate) fn get(&self, id: MeshId) -> Option<&MeshDesc> {
        self.storage.get(id)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (MeshId, &MeshDesc)> {
        self.storage.iter()
    }
}
