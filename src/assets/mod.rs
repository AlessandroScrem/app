

pub(crate) mod file;
pub(crate) mod gltf_loader;
pub(crate) mod image_decoder;
pub(crate) mod vertexdata;

pub(crate) mod texture_upload;


pub(crate) use crate::assets::vertexdata::MeshVertexData;
pub(crate) use crate::prelude::*;
pub(crate) mod material_desc;
pub(crate) mod texture_asset;
pub(crate) mod material_asset;
pub(crate) mod mesh_asset;
pub(crate) mod ibl_asset;
pub(crate) mod global_asset_manager;

pub use assets::global_asset_manager::resource_stats::ResourceStats;


pub type MeshId = assets::global_asset_manager::GlobalAssetId;
pub type MaterialId = assets::global_asset_manager::GlobalAssetId;
pub type TextureId = assets::global_asset_manager::GlobalAssetId;

// implementazione Display
impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
