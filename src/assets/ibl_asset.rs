use std::path::PathBuf;

use crate::assets::asset_manager::Asset;
use crate::ResourceId;

#[derive(Default)]
pub struct IblAsset {
    pub path: PathBuf,
    pub hrd_id: ResourceId,
}

impl IblAsset {
    pub fn new(hdr_id: ResourceId, path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            hrd_id: hdr_id,
        }
    }
}

impl Asset for IblAsset {
    type Key = std::path::PathBuf;
    fn key(&self) -> &Self::Key {
        &self.path
    }

    fn dependencies(&self) -> Vec<ResourceId> {
        vec![self.hrd_id]
    }
}
