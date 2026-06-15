use crate::assets::material_desc::{MaterialDesc};
use crate::assets::global_asset_manager::asset_storage::Asset;
use crate::assets::ResourceStats;
use crate::gpu::global_asset_manager::GlobalAssetId;


///////////////////////////////
// MATERIAL
///////////////////////////////
#[derive(Clone)]
pub struct MaterialAsset {
    pub stats: ResourceStats,
    pub desc: MaterialDesc,
}

impl Asset for MaterialAsset {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.desc.name
    }
    fn dependencies(&self) -> Vec<GlobalAssetId> {
        self.desc.get_textures()
    }
}