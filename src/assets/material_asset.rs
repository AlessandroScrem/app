use crate::assets::material_desc::{MaterialDesc};
use crate::assets::global_asset_manager::asset_storage::Asset;
use crate::gpu::global_asset_manager::GlobalAssetId;


#[derive(Clone)]
pub struct MaterialAsset {
    pub desc: MaterialDesc,
    pub key: String,
}

impl Asset for MaterialAsset {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.key
    }
    fn dependencies(&self) -> Vec<GlobalAssetId> {
        self.desc.get_textures()
    }
    fn estimated_size(&self) -> usize {
        MaterialDesc::estimated_size()
    }
}