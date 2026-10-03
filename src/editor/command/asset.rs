use std::path::PathBuf;

use crate::assets::{MaterialId, material_desc::MaterialDesc};

#[derive(Clone, Debug)]
pub enum AssetCommand {
    LoadGltf(PathBuf),
    AddIbl(PathBuf),
    UpdateMaterial {
        id: MaterialId,
        desc: MaterialDesc,
    },
}

impl AssetCommand {
    pub fn settings_changed(&self) -> bool {
        matches!(self, Self::AddIbl { .. })
    }
}
