

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
pub(crate) mod asset_manager;

pub (crate) use self::texture_asset::TextureAsset;
pub (crate) use self::material_asset::MaterialAsset;
pub (crate) use self::mesh_asset::MeshAsset;
pub (crate) use self::ibl_asset::IblAsset;
pub (crate) use self::asset_manager::resource_stats::ResourceStats;


pub type MeshId = assets::asset_manager::GlobalAssetId;
pub type MaterialId = assets::asset_manager::GlobalAssetId;
pub type TextureId = assets::asset_manager::GlobalAssetId;

// implementazione Display
impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
