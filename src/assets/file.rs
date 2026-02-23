use std::path::Path;

pub(crate) fn read_bytes<P: AsRef<Path>>(filepath: P) -> Result<Vec<u8>, String> {
    let path = filepath.as_ref();
    std::fs::read(path)
        .map_err(|_| format!("File Error: unable to load {}", path.to_string_lossy()))
}
