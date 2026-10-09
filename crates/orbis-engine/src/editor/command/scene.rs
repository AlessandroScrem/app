use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum SceneCommand {
    Open(PathBuf),
    Save,
    SaveAs(PathBuf),
    Clear,
}
