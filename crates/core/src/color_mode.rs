//! Color mode definitions for OverSprite

use crate::Rgba;
use serde::{Deserialize, Serialize};

/// Color mode of the sprite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    /// Indexed color (8-bit palette index per pixel)
    Indexed,
    /// Grayscale (16-bit: value + alpha)
    Grayscale,
    /// RGBA color (32-bit: red, green, blue, alpha)
    Rgba,
    /// Grayscale with alpha (16-bit: value + alpha) - alias for Grayscale
    GrayscaleAlpha,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Rgba
    }
}

impl ColorMode {
    /// Bits per pixel for this color mode
    pub fn bits_per_pixel(&self) -> u8 {
        match self {
            Self::Indexed => 8,
            Self::Grayscale | Self::GrayscaleAlpha => 16,
            Self::Rgba => 32,
        }
    }

    /// Bytes per pixel for this color mode
    pub fn bytes_per_pixel(&self) -> usize {
        (self.bits_per_pixel() / 8) as usize
    }

    /// Maximum number of palette entries (for indexed mode)
    pub fn max_palette_size(&self) -> usize {
        match self {
            Self::Indexed => 65536,
            _ => 0,
        }
    }

    /// Whether this mode uses a palette
    pub fn uses_palette(&self) -> bool {
        matches!(self, Self::Indexed)
    }

    /// Whether this mode has alpha channel
    pub fn has_alpha(&self) -> bool {
        matches!(self, Self::Rgba | Self::Grayscale | Self::GrayscaleAlpha)
    }

    /// Convert to Aseprite color depth value
    pub fn to_aseprite_depth(&self) -> u16 {
        match self {
            Self::Indexed => 8,
            Self::Grayscale | Self::GrayscaleAlpha => 16,
            Self::Rgba => 32,
        }
    }

    /// Create from Aseprite color depth
    pub fn from_aseprite_depth(depth: u16) -> Option<Self> {
        match depth {
            8 => Some(Self::Indexed),
            16 => Some(Self::Grayscale),
            32 => Some(Self::Rgba),
            _ => None,
        }
    }
}

/// Trait for pixel formats
pub trait PixelFormat: Sized + Copy + Default {
    fn color_mode() -> ColorMode;
    fn to_rgba(&self) -> Rgba;
    fn from_rgba(rgba: Rgba) -> Self;
    fn bytes_per_pixel() -> usize;
}

/// RGBA pixel (32-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RgbaPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PixelFormat for RgbaPixel {
    fn color_mode() -> ColorMode {
        ColorMode::Rgba
    }

    fn to_rgba(&self) -> Rgba {
        Rgba::new(self.r, self.g, self.b, self.a)
    }

    fn from_rgba(rgba: Rgba) -> Self {
        Self {
            r: rgba.r,
            g: rgba.g,
            b: rgba.b,
            a: rgba.a,
        }
    }

    fn bytes_per_pixel() -> usize {
        4
    }
}

/// Grayscale pixel (16-bit: value + alpha)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GrayscalePixel {
    pub v: u8,
    pub a: u8,
}

impl PixelFormat for GrayscalePixel {
    fn color_mode() -> ColorMode {
        ColorMode::Grayscale
    }

    fn to_rgba(&self) -> Rgba {
        Rgba::new(self.v, self.v, self.v, self.a)
    }

    fn from_rgba(rgba: Rgba) -> Self {
        // Convert to grayscale using luminance formula
        let v = ((rgba.r as u32 * 299 + rgba.g as u32 * 587 + rgba.b as u32 * 114) / 1000) as u8;
        Self {
            v,
            a: rgba.a,
        }
    }

    fn bytes_per_pixel() -> usize {
        2
    }
}

/// Indexed pixel (8-bit palette index)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IndexedPixel {
    pub index: u8,
}

impl PixelFormat for IndexedPixel {
    fn color_mode() -> ColorMode {
        ColorMode::Indexed
    }

    fn to_rgba(&self) -> Rgba {
        // Requires palette lookup - returns transparent as placeholder
        Rgba::TRANSPARENT
    }

    fn from_rgba(_rgba: Rgba) -> Self {
        // Requires palette quantization - returns 0 as placeholder
        Self { index: 0 }
    }

    fn bytes_per_pixel() -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_properties() {
        assert_eq!(ColorMode::Indexed.bits_per_pixel(), 8);
        assert_eq!(ColorMode::Grayscale.bits_per_pixel(), 16);
        assert_eq!(ColorMode::Rgba.bits_per_pixel(), 32);

        assert!(ColorMode::Indexed.uses_palette());
        assert!(!ColorMode::Rgba.uses_palette());

        assert!(ColorMode::Rgba.has_alpha());
        assert!(ColorMode::Grayscale.has_alpha());
        assert!(!ColorMode::Indexed.has_alpha()); // Alpha in palette, not pixel
    }

    #[test]
    fn test_rgba_pixel() {
        let pixel = RgbaPixel::from_rgba(Rgba::new(255, 128, 64, 200));
        assert_eq!(pixel.r, 255);
        assert_eq!(pixel.g, 128);
        assert_eq!(pixel.b, 64);
        assert_eq!(pixel.a, 200);

        let rgba = pixel.to_rgba();
        assert_eq!(rgba.r, 255);
        assert_eq!(rgba.g, 128);
        assert_eq!(rgba.b, 64);
        assert_eq!(rgba.a, 200);
    }

    #[test]
    fn test_grayscale_conversion() {
        let pixel = GrayscalePixel::from_rgba(Rgba::new(255, 255, 255, 255));
        assert_eq!(pixel.v, 255);

        let pixel = GrayscalePixel::from_rgba(Rgba::new(0, 0, 0, 255));
        assert_eq!(pixel.v, 0);
    }
}