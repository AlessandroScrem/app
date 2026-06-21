use crate::assets::asset_manager::asset_storage::Asset;
use crate::assets::asset_manager::GlobalAssetId;

use std::path::PathBuf;

use crate::renderer::MeshVertexData;
use crate::BoundingBox;

#[derive(Default)]
pub struct MeshDesc {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
    pub bounds: BoundingBox,
}

impl MeshDesc {
    pub fn get_materials(&self) ->Vec<GlobalAssetId> {
        self.submeshes.iter().map(|sm| sm.material).collect()
    }

    /// estimete size in bytes
    pub fn estimated_size(&self) -> usize {
        self.vertices.len() * size_of::<MeshVertexData>() + self.indices.len() * size_of::<u32>()
    }
}


pub struct SubMesh {
    pub index_range: std::ops::Range<u32>,
    pub material: GlobalAssetId,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum MeshSource {
    File {
        path: PathBuf,
        submesh_index: usize, // submesh index nel file gltf
    },
}

pub struct MeshAsset {
    pub desc: MeshDesc,
    pub mesh_source: MeshSource,
}

impl Asset for MeshAsset {
    type Key = MeshSource;

    fn key(&self) -> &Self::Key {
        &self.mesh_source
    }

    fn dependencies(&self) -> Vec<GlobalAssetId> {
        self.desc.get_materials()
    }

    fn estimated_size(&self) -> usize {
        self.desc.estimated_size()
    }
}