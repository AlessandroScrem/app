
mod file;
mod image_decoder;
mod vertexdata;

pub(crate) mod asset_manager;
pub(crate) mod gltf_loader;
pub(crate) mod ibl_asset;
pub(crate) mod material_asset;
pub(crate) mod material_desc;
pub(crate) mod mesh_asset;
pub(crate) mod texture_asset;
pub(crate) mod texture_upload;

pub(crate) use self::ibl_asset::IblAsset;
pub(crate) use self::material_asset::MaterialAsset;
pub(crate) use self::mesh_asset::MeshAsset;
pub(crate) use self::texture_asset::TextureAsset;
pub(crate) use vertexdata::*;

pub(crate) use self::asset_manager::GlobalAssetId;
pub(crate) type MeshId = crate::assets::asset_manager::GlobalAssetId;
pub(crate) type MaterialId = crate::assets::asset_manager::GlobalAssetId;
pub(crate) type TextureId = crate::assets::asset_manager::GlobalAssetId;
pub(crate) type IblId = crate::assets::asset_manager::GlobalAssetId;

// implementazione Display
impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

crate::impl_debug_drop!(TextureAsset);
crate::impl_debug_drop!(MeshAsset);
crate::impl_debug_drop!(MaterialAsset);
