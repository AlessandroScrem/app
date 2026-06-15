// use std::{collections::HashMap, path::PathBuf};

// use slotmap::SlotMap;
// use slotmap::new_key_type;

pub(crate) mod file;
pub(crate) mod gltf_loader;
pub(crate) mod image_decoder;
pub(crate) mod vertexdata;

pub(crate) mod texture_upload;
// pub(crate) mod asset_manager;
// pub(crate) mod material_asset;
// pub(crate) mod texture_asset;
// pub(crate) mod material_pbr;
// pub(crate) mod mesh_asset;


// pub(crate) use material_asset::*;
// pub(crate) use material_pbr::*;
// pub(crate) use texture_asset::*;
// pub(crate) use mesh_asset::*;
// pub(crate) use asset_manager::*;

pub(crate) use crate::assets::vertexdata::MeshVertexData;
pub(crate) use crate::prelude::*;
pub(crate) mod material_desc;
pub(crate) mod texture_asset;
pub(crate) mod material_asset;
pub(crate) mod mesh_asset;
pub(crate) mod ibl_asset;
pub(crate) mod global_asset_manager;

pub use assets::global_asset_manager::resource_stats::ResourceStats;


// new_key_type! {
    //     pub struct TextureId;
    //     pub struct MaterialId;
//     pub struct MeshId;
// }

pub type MeshId = assets::global_asset_manager::GlobalAssetId;
pub type MaterialId = assets::global_asset_manager::GlobalAssetId;
pub type TextureId = assets::global_asset_manager::GlobalAssetId;

// implementazione Display
impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // MaterialId è un wrapper int interno
        write!(f, "{:?}", self)
    }
}
