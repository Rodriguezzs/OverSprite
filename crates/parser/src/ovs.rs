//! OverSprite (.ovs) file format parser
//!
//! Parses and writes .ovs files according to the SPEC.md specification.
//! The .ovs format is a ZIP container with MessagePack-encoded metadata
//! and LZ4-compressed pixel data.

pub mod reader;
pub mod writer;

pub use self::reader::OvsReader;
pub use self::writer::OvsWriter;

use oversprite_core::{CoreResult, Sprite};
use std::path::Path;

/// Read a .ovs file
pub fn read_ovs(path: &Path) -> CoreResult<Sprite> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    OvsReader::new().read(&mut reader)
}

/// Write a sprite to .ovs format
pub fn write_ovs(sprite: &Sprite, path: &Path) -> CoreResult<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    OvsWriter::new().write(sprite, &mut writer)
}

/// Magic number identifying the `.ovs` format.
///
/// Stored as a 4-byte little-endian value in the manifest header and
/// verified on read, so an `.ovs` file can never be confused with a
/// renamed `.aseprite` or an unrelated ZIP container.
pub const OVS_MAGIC: u32 = 0x5356_4F01; // "SVO\x01" little-endian

/// Format identifier written to the manifest header.
pub const OVS_FORMAT_ID: &str = "ovs";

/// Ovs format constants
pub const OVS_MANIFEST: &str = "manifest.msgpack";
pub const OVS_METADATA: &str = "metadata.msgpack";
pub const OVS_LAYERS_INDEX: &str = "layers/index.msgpack";
pub const OVS_LAYERS_DIR: &str = "layers/";
pub const OVS_FRAMES_INDEX: &str = "frames/index.msgpack";
pub const OVS_FRAMES_DIR: &str = "frames/";
pub const OVS_PALETTE_GLOBAL: &str = "palette/global.msgpack";
pub const OVS_PALETTE_DIR: &str = "palette/";
pub const OVS_TAGS_INDEX: &str = "tags/index.msgpack";
pub const OVS_SLICES_INDEX: &str = "slices/index.msgpack";
pub const OVS_USER_DATA_DIR: &str = "user-data/";
pub const OVS_THUMBNAILS_DIR: &str = "thumbnails/";
pub const OVS_ASSETS_DIR: &str = "assets/";

/// Compression methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OvsCompression {
    None,
    Deflate,
    Lz4,
    Zstd,
}

impl Default for OvsCompression {
    fn default() -> Self {
        Self::Lz4
    }
}