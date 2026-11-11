//! Pixel type definitions for OverSprite

use crate::{Rgba, RgbaPixel, GrayscalePixel, IndexedPixel, PixelFormat};
use serde::{Deserialize, Serialize};

/// Pixel data wrapper for different color modes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Pixel {
    Rgba(RgbaPixel),
    Grayscale(GrayscalePixel),
    Indexed(IndexedPixel),
}

impl Pixel {
    pub fn to_rgba(&self, palette: Option<&crate::Palette>) -> Rgba {
        match self {
            Pixel::Rgba(p) => p.to_rgba(),
            Pixel::Grayscale(p) => p.to_rgba(),
            Pixel::Indexed(p) => {
                if let Some(pal) = palette {
                    pal.get(p.index.into()).map(|c| c.to_rgba()).unwrap_or(Rgba::TRANSPARENT)
                } else {
                    Rgba::TRANSPARENT
                }
            }
        }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Pixel::Rgba(_) => 4,
            Pixel::Grayscale(_) => 2,
            Pixel::Indexed(_) => 1,
        }
    }
}