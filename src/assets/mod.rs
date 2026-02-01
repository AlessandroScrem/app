use std::{collections::HashMap, path::PathBuf};

use slotmap::SlotMap;
use slotmap::new_key_type;

use crate::BoundingBox;

pub mod asset_manager;
pub mod file;
pub mod gltf_loader;
pub mod material_asset;
pub mod texture_asset;
pub mod mesh_asset;
pub mod vertexdata;

new_key_type! {
    pub struct TextureId;
    pub struct MaterialId;
    pub struct MeshId;
}

pub use crate::assets::vertexdata::MeshVertexData;
pub use texture_asset::*;
pub use mesh_asset::*;
pub use material_asset::*;




