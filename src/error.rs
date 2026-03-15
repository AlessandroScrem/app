use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("{0}")]
    Message(String),
    // Io(std::io::Error),
    // Image(image::ImageError),
    // FallbackWhite,
    // DecodeError,
}

impl From<String> for TextureError {
    fn from(value: String) -> Self {
        TextureError::Message(value)
    }
}

#[derive(Debug)]
pub enum ImportError {
    Io(std::io::Error),
    Gltf(gltf::Error),
    MissingPositions,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::MissingPositions => write!(f, "Missing vertices positions"),
            ImportError::Io(e) => write!(f, "IO error: {}", e),
            ImportError::Gltf(e) => write!(f, "glTF error: {}", e),
        }
    }
}