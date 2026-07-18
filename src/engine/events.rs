use std::path::PathBuf;

pub enum RuntimeEvent {
    Resize { width: u32, height: u32 },
    CloseRequested,
    DroppedFile(PathBuf),
    SetWindowTitle(String),
}
