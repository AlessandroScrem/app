use std::path::PathBuf;


#[derive(Clone, Debug)]
pub enum AssetCommand {
    LoadGltf(PathBuf),
    AddIbl(PathBuf),
}

impl AssetCommand {
    pub fn settings_changed(&self) -> bool {
        matches!(self, Self::AddIbl { .. })
    }
}
