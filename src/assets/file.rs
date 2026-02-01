use std::path::Path;

pub fn read_bytes<P: AsRef<Path>>(filepath: P) -> Option<Vec<u8>> {
    match std::fs::read(filepath) {
        Ok(buffer) => {
            // println!("read filepath {} ", filepath.display());
            Some(buffer)
        }
        Err(_err) => {
            // info!(
            //     "{}, Impossibile leggere il file {}",
            //     err,
            //     filepath.display()
            // );
            None
        }
    }
}
