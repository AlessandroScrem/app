use std::path::PathBuf;

// use crate::assets::{material_desc::MaterialDesc, MaterialId};

#[derive(Clone, Debug)]
pub enum AssetCommand {
    LoadGltf(PathBuf),
    AddIbl(PathBuf),
    // UpdateMaterial {
    //     material: MaterialId,
    //     desc: MaterialDesc,
    // },
}

impl AssetCommand {
    pub fn settings_changed(&self) -> bool {
        matches!(self, Self::AddIbl { .. })
    }
}