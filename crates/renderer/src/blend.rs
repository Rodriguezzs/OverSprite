//! OverSprite Renderer - Blend Mode Module
//!
//! Implements all Aseprite blend modes with tiny-skia equivalents.

use oversprite_core::BlendMode;
use tiny_skia::BlendMode as TinySkiaBlendMode;

/// Extension trait for converting Aseprite blend modes to tiny-skia
pub trait BlendModeExt {
    fn to_tiny_skia(&self) -> TinySkiaBlendMode;
}

impl BlendModeExt for BlendMode {
    fn to_tiny_skia(&self) -> TinySkiaBlendMode {
        match self {
            // Normal group
            BlendMode::Normal => TinySkiaBlendMode::SourceOver,
            BlendMode::Dissolve => TinySkiaBlendMode::SourceOver, // Approximation - tiny-skia doesn't have dissolve

            // Darken group
            BlendMode::Darken => TinySkiaBlendMode::Darken,
            BlendMode::Multiply => TinySkiaBlendMode::Multiply,
            BlendMode::ColorBurn => TinySkiaBlendMode::ColorBurn,
            BlendMode::LinearBurn => TinySkiaBlendMode::Darken, // Approximation
            BlendMode::DarkerColor => TinySkiaBlendMode::Darken,

            // Lighten group
            BlendMode::Lighten => TinySkiaBlendMode::Lighten,
            BlendMode::Screen => TinySkiaBlendMode::Screen,
            BlendMode::ColorDodge => TinySkiaBlendMode::ColorDodge,
            BlendMode::LinearDodge => TinySkiaBlendMode::Plus, // Add
            BlendMode::LighterColor => TinySkiaBlendMode::Lighten,

            // Contrast group
            BlendMode::Overlay => TinySkiaBlendMode::Overlay,
            BlendMode::SoftLight => TinySkiaBlendMode::SoftLight,
            BlendMode::HardLight => TinySkiaBlendMode::HardLight,
            BlendMode::VividLight => TinySkiaBlendMode::Overlay, // Approximation
            BlendMode::LinearLight => TinySkiaBlendMode::Overlay, // Approximation
            BlendMode::PinLight => TinySkiaBlendMode::Overlay, // Approximation
            BlendMode::HardMix => TinySkiaBlendMode::Overlay, // Approximation

            // Inversion group
            BlendMode::Difference => TinySkiaBlendMode::Difference,
            BlendMode::Exclusion => TinySkiaBlendMode::Exclusion,
            BlendMode::Subtract => TinySkiaBlendMode::SourceOver, // Custom needed - not directly supported
            BlendMode::Divide => TinySkiaBlendMode::SourceOver, // Custom needed - not directly supported

            // Component group
            BlendMode::Hue => TinySkiaBlendMode::Hue,
            BlendMode::Saturation => TinySkiaBlendMode::Saturation,
            BlendMode::Color => TinySkiaBlendMode::Color,
            BlendMode::Luminosity => TinySkiaBlendMode::Luminosity,

            // Additional
            BlendMode::Addition => TinySkiaBlendMode::Plus,
            BlendMode::Subtraction => TinySkiaBlendMode::SourceOver,
            BlendMode::SourceAtop => TinySkiaBlendMode::SourceAtop,
            BlendMode::AlphaDarken => TinySkiaBlendMode::Darken,
        }
    }
}

/// Convert Aseprite BlendMode to tiny-skia BlendMode (standalone function)
pub fn to_tiny_skia_blend_mode(mode: BlendMode) -> TinySkiaBlendMode {
    mode.to_tiny_skia()
}

/// Custom blend mode implementations for modes not directly supported by tiny-skia
/// These would need custom shaders or pixel-level operations
pub mod custom {
    use tiny_skia::{Pixmap, PremultipliedColorU8};

    /// Apply subtract blend manually (dst - src)
    pub fn subtract_blend(dst: &mut Pixmap, src: &Pixmap, opacity: f32) {
        let dst_pixels = dst.pixels_mut();
        let src_pixels = src.pixels();

        for (dst_pixel, src_pixel) in dst_pixels.iter_mut().zip(src_pixels.iter()) {
            let src_premul = src_pixel;

            let r = (dst_pixel.red() as f32 - src_premul.red() as f32 * opacity).max(0.0) as u8;
            let g = (dst_pixel.green() as f32 - src_premul.green() as f32 * opacity).max(0.0) as u8;
            let b = (dst_pixel.blue() as f32 - src_premul.blue() as f32 * opacity).max(0.0) as u8;
            let a = dst_pixel.alpha();

            *dst_pixel = PremultipliedColorU8::from_rgba(r, g, b, a)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
        }
    }

    /// Apply divide blend manually (dst / src)
    pub fn divide_blend(dst: &mut Pixmap, src: &Pixmap, opacity: f32) {
        let dst_pixels = dst.pixels_mut();
        let src_pixels = src.pixels();

        for (dst_pixel, src_pixel) in dst_pixels.iter_mut().zip(src_pixels.iter()) {
            let src_premul = src_pixel;

            let src_r = (src_premul.red() as f32 / 255.0).max(0.001);
            let src_g = (src_premul.green() as f32 / 255.0).max(0.001);
            let src_b = (src_premul.blue() as f32 / 255.0).max(0.001);

            let r = (dst_pixel.red() as f32 / src_r * opacity).min(255.0) as u8;
            let g = (dst_pixel.green() as f32 / src_g * opacity).min(255.0) as u8;
            let b = (dst_pixel.blue() as f32 / src_b * opacity).min(255.0) as u8;
            let a = dst_pixel.alpha();

            *dst_pixel = PremultipliedColorU8::from_rgba(r, g, b, a)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
        }
    }

    /// Apply linear burn blend manually (dst + src - 1)
    pub fn linear_burn_blend(dst: &mut Pixmap, src: &Pixmap, opacity: f32) {
        let dst_pixels = dst.pixels_mut();
        let src_pixels = src.pixels();

        for (dst_pixel, src_pixel) in dst_pixels.iter_mut().zip(src_pixels.iter()) {
            let src_premul = src_pixel;

            let r = (dst_pixel.red() as f32 + src_premul.red() as f32 * opacity - 255.0 * opacity).max(0.0) as u8;
            let g = (dst_pixel.green() as f32 + src_premul.green() as f32 * opacity - 255.0 * opacity).max(0.0) as u8;
            let b = (dst_pixel.blue() as f32 + src_premul.blue() as f32 * opacity - 255.0 * opacity).max(0.0) as u8;
            let a = dst_pixel.alpha();

            *dst_pixel = PremultipliedColorU8::from_rgba(r, g, b, a)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
        }
    }

    /// Apply linear dodge (add) blend manually
    pub fn linear_dodge_blend(dst: &mut Pixmap, src: &Pixmap, opacity: f32) {
        let dst_pixels = dst.pixels_mut();
        let src_pixels = src.pixels();

        for (dst_pixel, src_pixel) in dst_pixels.iter_mut().zip(src_pixels.iter()) {
            let src_premul = src_pixel;

            let r = (dst_pixel.red() as f32 + src_premul.red() as f32 * opacity).min(255.0) as u8;
            let g = (dst_pixel.green() as f32 + src_premul.green() as f32 * opacity).min(255.0) as u8;
            let b = (dst_pixel.blue() as f32 + src_premul.blue() as f32 * opacity).min(255.0) as u8;
            let a = dst_pixel.alpha();

            *dst_pixel = PremultipliedColorU8::from_rgba(r, g, b, a)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
        }
    }
}

/// Check if this blend mode requires custom implementation (not directly supported by tiny-skia)
pub fn requires_custom_impl(mode: BlendMode) -> bool {
    matches!(mode,
        BlendMode::Subtract |
        BlendMode::Divide |
        BlendMode::LinearBurn |
        BlendMode::LinearDodge |
        BlendMode::VividLight |
        BlendMode::LinearLight |
        BlendMode::PinLight |
        BlendMode::HardMix |
        BlendMode::Subtraction |
        BlendMode::Dissolve
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_tiny_skia() {
        assert_eq!(BlendMode::Normal.to_tiny_skia(), TinySkiaBlendMode::SourceOver);
        assert_eq!(BlendMode::Multiply.to_tiny_skia(), TinySkiaBlendMode::Multiply);
        assert_eq!(BlendMode::Screen.to_tiny_skia(), TinySkiaBlendMode::Screen);
    }

    #[test]
    fn test_requires_custom_impl() {
        assert!(requires_custom_impl(BlendMode::Subtract));
        assert!(requires_custom_impl(BlendMode::Divide));
        assert!(requires_custom_impl(BlendMode::LinearBurn));
        assert!(!requires_custom_impl(BlendMode::Normal));
        assert!(!requires_custom_impl(BlendMode::Multiply));
    }
}