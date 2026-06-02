use std::{collections::HashMap, path::PathBuf};

use slotmap::SlotMap;
use slotmap::new_key_type;

pub(crate) mod asset_manager;
pub(crate) mod file;
pub(crate) mod gltf_loader;
pub(crate) mod image_decoder;
pub(crate) mod material_asset;
pub(crate) mod material_pbr;
pub(crate) mod mesh_asset;
pub(crate) mod texture_asset;
pub(crate) mod texture_upload;
pub(crate) mod vertexdata;

pub(crate) mod global_asset_manager;


pub(crate) use crate::assets::vertexdata::MeshVertexData;
pub(crate) use crate::prelude::*;
pub(crate) use asset_manager::*;
pub(crate) use material_asset::*;
pub(crate) use material_pbr::*;
pub(crate) use mesh_asset::*;
pub(crate) use texture_asset::*;


new_key_type! {
    pub struct TextureId;
    pub struct MaterialId;
    pub struct MeshId;
}

// implementazione Display
impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // MaterialId è un wrapper int interno
        write!(f, "{:?}", self)
    }
}
