use std::path::PathBuf;

pub enum FileFilter {
    Gltf,
    Json,
    Hdr,
}

impl FileFilter {
    fn as_args(&self) -> (&str, &[&str]) {
        match self {
            Self::Gltf => ("glTF", &["gltf", "glb"]),
            Self::Json => ("json", &["json"]),
            Self::Hdr => ("hdr", &["hdr"]),
        }
    }
}

pub fn file_save(filter: FileFilter) -> Option<PathBuf> {
    let (name, ext) = filter.as_args();
    rfd::FileDialog::new().add_filter(name, ext).save_file()
}

pub fn file_open(filter: FileFilter) -> Option<PathBuf> {
    let (name, ext) = filter.as_args();
    rfd::FileDialog::new().add_filter(name, ext).pick_file()
}
