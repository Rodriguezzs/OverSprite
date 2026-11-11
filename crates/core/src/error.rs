//! Core error types for OverSprite

use thiserror::Error;

/// Core crate errors
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid color mode: {0}")]
    InvalidColorMode(String),

    #[error("Invalid blend mode: {0}")]
    InvalidBlendMode(u8),

    #[error("Invalid layer type: {0}")]
    InvalidLayerType(String),

    #[error("Invalid cel type: {0}")]
    InvalidCelType(String),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Frame not found: {0}")]
    FrameNotFound(String),

    #[error("Cel not found: {0}")]
    CelNotFound(String),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Slice not found: {0}")]
    SliceNotFound(String),

    #[error("Tileset not found: {0}")]
    TilesetNotFound(String),

    #[error("Palette index out of bounds: {index} >= {max}")]
    PaletteIndexOutOfBounds { index: usize, max: usize },

    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Pixel data size mismatch: expected {expected}, got {actual}")]
    PixelDataSizeMismatch { expected: usize, actual: usize },

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("LZ4 error: {0}")]
    Lz4Error(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for core operations
pub type CoreResult<T> = Result<T, CoreError>;

impl From<rmp_serde::encode::Error> for CoreError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for CoreError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        Self::DeserializationError(err.to_string())
    }
}

impl From<lz4_flex::frame::Error> for CoreError {
    fn from(err: lz4_flex::frame::Error) -> Self {
        Self::Lz4Error(err.to_string())
    }
}

impl From<uuid::Error> for CoreError {
    fn from(err: uuid::Error) -> Self {
        Self::InvalidUuid(err.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        Self::DeserializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CoreError::InvalidColorMode("unknown".to_string());
        assert_eq!(err.to_string(), "Invalid color mode: unknown");
    }

    #[test]
    fn test_palette_index_error() {
        let err = CoreError::PaletteIndexOutOfBounds { index: 256, max: 256 };
        assert_eq!(err.to_string(), "Palette index out of bounds: 256 >= 256");
    }
}