//! Parser error types for OverSprite

use thiserror::Error;

/// Parser errors
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Unexpected EOF")]
    UnexpectedEof,

    #[error("Invalid magic number: expected {expected:04X}, got {actual:04X}")]
    InvalidMagic { expected: u16, actual: u16 },

    #[error("Invalid .ovs magic number: expected {expected:08X}, got {actual:08X}")]
    InvalidOvsMagic { expected: u32, actual: u32 },

    #[error("Invalid chunk type: {0:04X}")]
    InvalidChunkType(u16),

    #[error("Chunk size mismatch: expected {expected}, got {actual}")]
    ChunkSizeMismatch { expected: usize, actual: usize },

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("LZ4 error: {0}")]
    Lz4Error(#[from] lz4_flex::frame::Error),

    #[error("MessagePack error: {0}")]
    MessagePackError(#[from] rmp_serde::decode::Error),

    #[error("MessagePack encode error: {0}")]
    MessagePackEncodeError(#[from] rmp_serde::encode::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Core error: {0}")]
    CoreError(#[from] oversprite_core::CoreError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Result type for parser operations
pub type ParserResult<T> = Result<T, ParserError>;

impl From<ParserError> for oversprite_core::CoreError {
    fn from(err: ParserError) -> Self {
        oversprite_core::CoreError::Internal(err.to_string())
    }
}