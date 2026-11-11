//! Metadata definitions for OverSprite

use crate::{ColorMode, ColorProfile, CoreError, CoreResult, FileFlags, Grid, Rgba, Size};
use serde::{Deserialize, Serialize};

/// Sprite metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub dimensions: Size,
    pub color_mode: ColorMode,
    pub color_depth: u16,
    pub transparent_index: u8,
    pub background_color: Rgba,
    pub pixel_aspect: PixelAspect,
    pub grid: Grid,
    pub color_profile: ColorProfile,
    pub flags: FileFlags,
    pub default_frame_duration_ms: u32,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            dimensions: Size::new(320, 180),
            color_mode: ColorMode::Rgba,
            color_depth: 32,
            transparent_index: 0,
            background_color: Rgba::TRANSPARENT,
            pixel_aspect: PixelAspect::square(),
            grid: Grid::default(),
            color_profile: ColorProfile::srgb(),
            flags: FileFlags::LAYER_OPACITY_VALID | FileFlags::LAYER_BLEND_MODE_VALID | FileFlags::LAYERS_HAVE_UUIDS,
            default_frame_duration_ms: 100,
        }
    }
}

impl Metadata {
    pub fn new(width: u32, height: u32, color_mode: ColorMode) -> Self {
        let mut meta = Self::default();
        meta.dimensions = Size::new(width, height);
        meta.color_mode = color_mode;
        meta.color_depth = color_mode.to_aseprite_depth();
        meta
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.dimensions.w == 0 || self.dimensions.h == 0 {
            return Err(CoreError::InvalidDimensions {
                width: self.dimensions.w,
                height: self.dimensions.h,
            });
        }
        if self.dimensions.w > 65535 || self.dimensions.h > 65535 {
            return Err(CoreError::ValidationError("Dimensions exceed maximum 65535".into()));
        }
        Ok(())
    }
}

/// Pixel aspect ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelAspect {
    pub width: u16,
    pub height: u16,
}

impl PixelAspect {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn square() -> Self {
        Self { width: 1, height: 1 }
    }

    pub fn ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let meta = Metadata::new(640, 480, ColorMode::Indexed);
        assert_eq!(meta.dimensions.w, 640);
        assert_eq!(meta.dimensions.h, 480);
        assert_eq!(meta.color_mode, ColorMode::Indexed);
        assert_eq!(meta.color_depth, 8);
    }

    #[test]
    fn test_pixel_aspect() {
        let aspect = PixelAspect::new(2, 1);
        assert_eq!(aspect.ratio(), 2.0);
    }
}