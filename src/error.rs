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