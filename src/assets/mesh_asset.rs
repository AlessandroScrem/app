use super::*;

pub struct MeshDesc {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
    pub bounds: BoundingBox,
}

pub struct SubMesh {
    pub index_range: std::ops::Range<u32>,
    pub base_vertex: u32,
    pub material: MaterialId,
}

#[derive(Hash, Eq, PartialEq)]
pub enum MeshSource {
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
pub enum Primitive {
    Cube,
    Quad,
    Sphere,
    Cylinder,
    Grid,
}

#[derive(Hash, Eq, PartialEq)]
pub struct MeshKey {
    pub source: MeshSource,
}

#[derive(Default)]
pub struct MeshAssets {
    storage: SlotMap<MeshId, MeshDesc>,
    lookup: HashMap<MeshKey, MeshId>,
}

impl MeshAssets {
    pub fn get_or_create(&mut self, key: MeshKey, desc_fn: impl FnOnce() -> MeshDesc) -> MeshId {
        if let Some(id) = self.lookup.get(&key) {
            return *id;
        }

        let desc = desc_fn();
        let id = self.storage.insert(desc);
        self.lookup.insert(key, id);
        id
    }

    pub fn remove(&mut self, id: MeshId) {
        if self.storage.contains_key(id) {
            self.storage.remove(id);
            self.lookup.retain(|_key, &mut id| id != id);
        }
    }

    pub fn contains_key(&self, id: MeshId) ->bool {
        self.storage.contains_key(id)
    }

    pub fn get(&self, id: MeshId) -> Option<&MeshDesc> {
        self.storage.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MeshId, &MeshDesc)> {
        self.storage.iter()
    }
}
