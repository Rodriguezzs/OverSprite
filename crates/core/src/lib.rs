//! OverSprite Core Data Model
//!
//! This crate defines the core data structures for the OverSprite (.ovs) format
//! as specified in SPEC.md. It provides the in-memory representation of sprites,
//! layers, frames, cels, palettes, and all related metadata.

pub mod blend_mode;
pub mod cel;
pub mod color_mode;
pub mod color_profile;
pub mod error;
pub mod frame;
pub mod grid;
pub mod layer;
pub mod metadata;
pub mod palette;
pub mod pixel;
pub mod slice;
pub mod sprite;
pub mod tag;
pub mod tileset;
pub mod user_data;
pub mod uuid_ext;

// Re-exports
pub use self::blend_mode::BlendMode;
pub use self::cel::{Cel, CelType, CelExtra};
pub use self::color_mode::{ColorMode, PixelFormat, RgbaPixel, GrayscalePixel, IndexedPixel};
pub use self::color_profile::{ColorProfile, ColorProfileType};
pub use self::error::{CoreError, CoreResult};
pub use self::frame::{Frame, FrameIndex};
pub use self::grid::Grid;
pub use self::layer::{Layer, LayerType, LayerIndex, LayerDepth};
pub use self::metadata::Metadata;
pub use self::palette::{Palette, PaletteColor, PaletteIndex};
pub use self::pixel::Pixel;
pub use self::slice::{Slice, SliceKey, Slice9Patch, Pivot};
pub use self::sprite::{Sprite, SpriteHeader, CompressionConfig};
pub use self::tag::Tag;
pub use self::tileset::{Tileset, TilesetFlags, TileData};
pub use self::user_data::{UserData, PropertyValue, PropertyMap};
pub use self::uuid_ext::UuidExt;
pub use uuid::Uuid;

use serde::{Deserialize, Serialize};

/// Fixed-point 16.16 representation for sub-pixel coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fixed16_16(pub f32);

impl Fixed16_16 {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub const fn from_int(value: i32) -> Self {
        Self(value as f32)
    }

    pub fn to_int(self) -> i32 {
        self.0 as i32
    }

    pub fn to_f32(self) -> f32 {
        self.0
    }
}

impl Default for Fixed16_16 {
    fn default() -> Self {
        Self(0.0)
    }
}

impl From<i32> for Fixed16_16 {
    fn from(value: i32) -> Self {
        Self::from_int(value)
    }
}

impl From<f32> for Fixed16_16 {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

/// Rectangle with integer coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn empty() -> Self {
        Self { x: 0, y: 0, w: 0, h: 0 }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + self.w as i32) &&
        y >= self.y && y < (self.y + self.h as i32)
    }
}

/// Point with integer coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// Size with unsigned dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }

    pub fn empty() -> Self {
        Self { w: 0, h: 0 }
    }

    pub fn area(&self) -> u64 {
        self.w as u64 * self.h as u64
    }
}

/// Color in RGBA format (8 bits per channel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl From<[u8; 4]> for Rgba {
    fn from(arr: [u8; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<[u8; 3]> for Rgba {
    fn from(arr: [u8; 3]) -> Self {
        Self::from_rgb(arr[0], arr[1], arr[2])
    }
}

/// Animation direction for tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationDirection {
    Forward,
    Reverse,
    PingPong,
    PingPongReverse,
}

impl Default for AnimationDirection {
    fn default() -> Self {
        Self::Forward
    }
}

/// Compression algorithm for pixel data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    None,
    Deflate,
    Lz4,
    Zstd,
}

impl Default for Compression {
    fn default() -> Self {
        Self::Lz4
    }
}

/// Features enabled in the file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    Tilesets,
    AnimationTags,
    Slices,
    UserData,
    Thumbnails,
    ExternalRefs,
}

bitflags::bitflags! {
    /// File-level flags matching Aseprite specification
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct FileFlags: u32 {
        const LAYER_OPACITY_VALID = 1;
        const LAYER_BLEND_MODE_VALID = 2;
        const LAYERS_HAVE_UUIDS = 4;
    }
}

bitflags::bitflags! {
    /// Layer flags matching Aseprite specification
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct LayerFlags: u16 {
        const VISIBLE = 1;
        const EDITABLE = 2;
        const LOCK_MOVEMENT = 4;
        const BACKGROUND = 8;
        const PREFER_LINKED_CELS = 16;
        const COLLAPSED = 32;
        const IS_REFERENCE = 64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_creation() {
        let c = Rgba::new(255, 128, 64, 200);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 200);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 100);
        assert!(rect.contains(10, 10));
        assert!(rect.contains(50, 50));
        assert!(rect.contains(109, 109));
        assert!(!rect.contains(110, 10));
        assert!(!rect.contains(10, 110));
    }

    #[test]
    fn test_fixed16_16() {
        let f = Fixed16_16::new(1.5);
        assert_eq!(f.to_f32(), 1.5);
        assert_eq!(f.to_int(), 1);
    }
}