//! OverSprite Parser
//!
//! This crate provides parsers for both .aseprite and .ovs (OverSprite) file formats.
//! It handles reading and writing sprite files with full compatibility.

pub mod aseprite;
pub mod ovs;
pub mod common;
pub mod error;

pub use self::aseprite::{read_aseprite, write_aseprite, AsepriteReader, AsepriteWriter};
pub use self::ovs::{read_ovs, write_ovs, OvsReader, OvsWriter, OVS_MAGIC, OVS_FORMAT_ID};
pub use self::error::{ParserError, ParserResult};

use oversprite_core::{CoreError, CoreResult, Sprite};

/// Read a sprite from either .aseprite or .ovs format (auto-detect)
pub fn read_sprite(path: &std::path::Path) -> CoreResult<Sprite> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "aseprite" | "ase" => read_aseprite(path),
        "ovs" => read_ovs(path),
        _ => Err(CoreError::UnsupportedFeature(format!("Unknown file extension: {}", ext))),
    }
}

/// Write a sprite to either .aseprite or .ovs format
pub fn write_sprite(sprite: &Sprite, path: &std::path::Path) -> CoreResult<()> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "aseprite" | "ase" => write_aseprite(sprite, path),
        "ovs" => write_ovs(sprite, path),
        _ => Err(CoreError::UnsupportedFeature(format!("Unknown file extension: {}", ext))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extension_detection() {
        assert_eq!(Path::new("test.aseprite").extension().unwrap(), "aseprite");
        assert_eq!(Path::new("test.ase").extension().unwrap(), "ase");
        assert_eq!(Path::new("test.ovs").extension().unwrap(), "ovs");
    }
}