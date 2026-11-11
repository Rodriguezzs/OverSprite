//! OverSprite Renderer - Onion Skin Module
//!
//! Implements onion skinning for animation preview with configurable
//! previous/next frame ranges, opacity falloff, and color tinting.

use oversprite_core::{CoreResult, Sprite, Frame, Cel, Rgba, CelType};
use crate::canvas::Canvas2D;
use crate::layer_renderer::LayerRenderer;
use tiny_skia::{BlendMode as TinySkiaBlendMode, Transform, Pixmap, PremultipliedColorU8};

/// Onion skin configuration
#[derive(Debug, Clone)]
pub struct OnionSkinConfig {
    pub enabled: bool,
    pub previous_frames: usize,
    pub next_frames: usize,
    pub opacity_falloff: f32,      // 0.0-1.0, how much opacity decreases per frame
    pub min_opacity: u8,           // Minimum opacity for farthest frames
    pub previous_tint: Rgba,       // Color tint for previous frames
    pub next_tint: Rgba,           // Color tint for next frames
    pub show_current_frame: bool,  // Whether to render current frame on top
}

impl Default for OnionSkinConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            previous_frames: 2,
            next_frames: 2,
            opacity_falloff: 0.5,
            min_opacity: 32,
            previous_tint: Rgba::new(255, 100, 100, 255),  // Red tint
            next_tint: Rgba::new(100, 255, 100, 255),     // Green tint
            show_current_frame: true,
        }
    }
}

/// Onion skin renderer
pub struct OnionSkinRenderer {
    pub config: OnionSkinConfig,
    layer_renderer: LayerRenderer,
}

impl Default for OnionSkinRenderer {
    fn default() -> Self {
        Self {
            config: OnionSkinConfig::default(),
            layer_renderer: LayerRenderer::new(),
        }
    }
}

impl OnionSkinRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: OnionSkinConfig) -> Self {
        Self { config, layer_renderer: LayerRenderer::new() }
    }

    /// Render onion skin for a frame
    pub fn render(
        &self,
        canvas: &mut Canvas2D,
        sprite: &Sprite,
        current_frame_index: usize,
    ) -> CoreResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let total_frames = sprite.frames.order.len();
        if total_frames <= 1 {
            return Ok(());
        }

        // Render previous frames (backwards in time)
        for i in 1..=self.config.previous_frames {
            let frame_index = if current_frame_index >= i {
                current_frame_index - i
            } else if self.config.show_current_frame {
                // Wrap around for ping-pong/loop
                total_frames - (i - current_frame_index)
            } else {
                continue;
            };

            let opacity = self.calculate_opacity(i, self.config.previous_frames);
            if opacity >= self.config.min_opacity {
                self.render_frame_with_tint(canvas, sprite, frame_index, opacity, self.config.previous_tint)?;
            }
        }

        // Render next frames (forward in time)
        for i in 1..=self.config.next_frames {
            let frame_index = (current_frame_index + i) % total_frames;
            let opacity = self.calculate_opacity(i, self.config.next_frames);
            if opacity >= self.config.min_opacity {
                self.render_frame_with_tint(canvas, sprite, frame_index, opacity, self.config.next_tint)?;
            }
        }

        // Render current frame on top if enabled
        if self.config.show_current_frame {
            self.layer_renderer.render_all_layers(canvas, sprite, current_frame_index)?;
        }

        Ok(())
    }

    fn calculate_opacity(&self, frame_distance: usize, max_frames: usize) -> u8 {
        let base_opacity = 255.0 * (1.0 - self.config.opacity_falloff);
        let falloff_per_frame = 255.0 * self.config.opacity_falloff / max_frames as f32;
        let opacity = base_opacity - falloff_per_frame * frame_distance as f32;
        opacity.clamp(self.config.min_opacity as f32, 255.0) as u8
    }

    fn render_frame_with_tint(
        &self,
        canvas: &mut Canvas2D,
        sprite: &Sprite,
        frame_index: usize,
        opacity: u8,
        tint: Rgba,
    ) -> CoreResult<()> {
        // tiny-skia doesn't have color matrix filters, so we apply tint manually per pixel
        // This is a simplified approach - we render the frame to an offscreen pixmap,
        // apply tint, then draw to main canvas

        // For now, we'll just render with the tinted opacity
        // A full implementation would need to apply the color matrix to each pixel

        if let Some(frame_uuid) = sprite.frames.order.get(frame_index) {
            if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                for layer_uuid in &sprite.layers.order {
                    if let Some(layer) = sprite.layer_data.get(layer_uuid) {
                        if layer.visible {
                            self.render_layer_tinted(canvas, sprite, layer, frame, opacity, tint)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn render_layer_tinted(
        &self,
        canvas: &mut Canvas2D,
        sprite: &Sprite,
        layer: &oversprite_core::Layer,
        frame: &Frame,
        opacity: u8,
        tint: Rgba,
    ) -> CoreResult<()> {
        if !layer.visible { return Ok(()); }

        for cel_uuid in &frame.cels {
            if let Some(cel) = sprite.cel_data.get(cel_uuid) {
                if cel.layer_uuid == layer.uuid {
                    self.render_cel_tinted(canvas, sprite, cel, opacity, tint)?;
                }
            }
        }
        Ok(())
    }

    fn render_cel_tinted(
        &self,
        canvas: &mut Canvas2D,
        sprite: &Sprite,
        cel: &Cel,
        opacity: u8,
        tint: Rgba,
    ) -> CoreResult<()> {
        if cel.cel_type == CelType::Raw || cel.cel_type == CelType::Compressed {
            let pixels = cel.user_data.get_property("pixels")
                .or_else(|| cel.user_data.get_property("raw_pixels"))
                .or_else(|| cel.user_data.get_property("compressed_pixels"));

            if let Some(pixels) = pixels {
                if let Some(bytes) = pixels.as_bytes() {
                    let image = LayerRenderer::create_image_from_bytes(
                        &bytes,
                        cel.bounds.w,
                        cel.bounds.h,
                        sprite.metadata.color_mode,
                    )?;

                    // Apply tint by creating a tinted version
                    let tinted = Self::apply_tint(&image, tint, opacity);

                    let transform = Transform::from_translate(cel.position.x as f32, cel.position.y as f32);
                    let paint = tiny_skia::PixmapPaint {
                        blend_mode: TinySkiaBlendMode::SourceOver,
                        opacity: opacity as f32 / 255.0,
                        ..Default::default()
                    };

                    canvas.pixmap.draw_pixmap(0, 0, tinted.as_ref(), &paint, transform, None);
                }
            }
        }
        Ok(())
    }

    /// Apply color tint to a pixmap
    fn apply_tint(pixmap: &Pixmap, tint: Rgba, opacity: u8) -> Pixmap {
        let mut result = Pixmap::new(pixmap.width(), pixmap.height()).unwrap();

        let tint_r = tint.r as f32 / 255.0;
        let tint_g = tint.g as f32 / 255.0;
        let tint_b = tint.b as f32 / 255.0;
        let tint_a = tint.a as f32 / 255.0;
        let opacity_f = opacity as f32 / 255.0;

        for y in 0..pixmap.height() {
            for x in 0..pixmap.width() {
                if let Some(pixel) = pixmap.pixel(x, y) {
                    // pixel is already PremultipliedColorU8
                    let r = pixel.red() as f32 / 255.0;
                    let g = pixel.green() as f32 / 255.0;
                    let b = pixel.blue() as f32 / 255.0;
                    let a = pixel.alpha() as f32 / 255.0;

                    // Apply tint (multiply)
                    let new_r = (r * tint_r * opacity_f * 255.0) as u8;
                    let new_g = (g * tint_g * opacity_f * 255.0) as u8;
                    let new_b = (b * tint_b * opacity_f * 255.0) as u8;
                    let new_a = (a * tint_a * opacity_f * 255.0) as u8;

                    let pixel_idx = (y * result.width() + x) as usize;
                    if let Some(out_pixel) = result.pixels_mut().get_mut(pixel_idx) {
                        *out_pixel = PremultipliedColorU8::from_rgba(new_r, new_g, new_b, new_a)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }

        result
    }
}

/// Onion skin mode for UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnionSkinMode {
    Off,
    Previous,
    Next,
    Both,
    Full, // All frames
}

impl OnionSkinMode {
    pub fn to_config(&self) -> OnionSkinConfig {
        match self {
            OnionSkinMode::Off => OnionSkinConfig { enabled: false, ..Default::default() },
            OnionSkinMode::Previous => OnionSkinConfig {
                enabled: true,
                previous_frames: 3,
                next_frames: 0,
                ..Default::default()
            },
            OnionSkinMode::Next => OnionSkinConfig {
                enabled: true,
                previous_frames: 0,
                next_frames: 3,
                ..Default::default()
            },
            OnionSkinMode::Both => OnionSkinConfig {
                enabled: true,
                previous_frames: 2,
                next_frames: 2,
                ..Default::default()
            },
            OnionSkinMode::Full => OnionSkinConfig {
                enabled: true,
                previous_frames: 5,
                next_frames: 5,
                opacity_falloff: 0.3,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onion_skin_config_default() {
        let config = OnionSkinConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.previous_frames, 2);
        assert_eq!(config.next_frames, 2);
    }

    #[test]
    fn test_opacity_calculation() {
        let renderer = OnionSkinRenderer::new();
        let opacity1 = renderer.calculate_opacity(1, 2);
        let opacity2 = renderer.calculate_opacity(2, 2);
        assert!(opacity1 > opacity2); // Closer frames more opaque
    }
}