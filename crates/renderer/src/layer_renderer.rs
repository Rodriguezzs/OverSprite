//! OverSprite Renderer - Layer Renderer Module
//!
//! Handles rendering of individual layers including image layers, tilemap layers, and groups.

use oversprite_core::{
    BlendMode, Cel, CelType, ColorMode, CoreError, CoreResult, Layer, LayerType, Palette, Point, Rgba, Sprite, Tileset, Uuid,
};
use tiny_skia::{Pixmap, PixmapPaint, Transform, PremultipliedColorU8};
use crate::blend::to_tiny_skia_blend_mode;
use std::collections::HashMap;

/// Layer renderer for drawing layers to a pixmap
pub struct LayerRenderer {
    /// Whether to render layer bounds for debugging
    pub debug_bounds: bool,
    /// Whether to render layer names for debugging
    pub debug_names: bool,
}

impl Default for LayerRenderer {
    fn default() -> Self {
        Self {
            debug_bounds: false,
            debug_names: false,
        }
    }
}

impl LayerRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a layer to the target pixmap
    pub fn render_layer(
        &self,
        target: &mut Pixmap,
        layer: &Layer,
        frame_cels: &[&Cel],
        cels: &HashMap<Uuid, Cel>,
        tilesets: &HashMap<Uuid, Tileset>,
        palette: &Palette,
        opacity_override: Option<u8>,
    ) -> CoreResult<()> {
        if !layer.visible {
            return Ok(());
        }

        let layer_opacity = opacity_override.unwrap_or(layer.opacity);
        if layer_opacity == 0 {
            return Ok(());
        }

        match layer.layer_type {
            LayerType::Image => {
                self.render_image_layer(target, layer, frame_cels, cels, palette, layer_opacity)?;
            }
            LayerType::Tilemap => {
                self.render_tilemap_layer(target, layer, frame_cels, cels, tilesets, palette, layer_opacity)?;
            }
            LayerType::Group => {
                // Groups don't render directly
            }
        }

        // Debug: draw layer bounds
        if self.debug_bounds {
            self.draw_layer_bounds(target, layer)?;
        }

        Ok(())
    }

    /// Render an image layer (composed of cels)
    fn render_image_layer(
        &self,
        target: &mut Pixmap,
        layer: &Layer,
        frame_cels: &[&Cel],
        _cels: &HashMap<Uuid, Cel>,
        palette: &Palette,
        layer_opacity: u8,
    ) -> CoreResult<()> {
        let blend_mode = to_tiny_skia_blend_mode(layer.blend_mode);

        // Render each cel in this layer for this frame
        for cel in frame_cels {
            if cel.layer_uuid != layer.uuid {
                continue;
            }

            // Get pixel data
            let pixels = cel.user_data.get_property("pixels")
                .or_else(|| cel.user_data.get_property("raw_pixels"))
                .or_else(|| cel.user_data.get_property("compressed_pixels"));

            let Some(pixels) = pixels else { continue };
            let Some(pixel_bytes) = pixels.as_bytes() else { continue };

            let width = cel.bounds.w;
            let height = cel.bounds.h;

            if width == 0 || height == 0 {
                continue;
            }

            // Create pixmap from pixel data
            let cel_pixmap = self.create_cel_pixmap(&pixel_bytes, width, height, palette, cel.opacity)?;

            // Draw with layer blend mode and combined opacity
            let combined_opacity = ((cel.opacity as u16 * layer_opacity as u16) / 255) as u8;
            let paint = PixmapPaint {
                blend_mode,
                opacity: combined_opacity as f32 / 255.0,
                ..Default::default()
            };

            let transform = Transform::from_translate(cel.position.x as f32, cel.position.y as f32);
            target.draw_pixmap(0, 0, cel_pixmap.as_ref(), &paint, transform, None);
        }

        Ok(())
    }

    /// Render a tilemap layer
    fn render_tilemap_layer(
        &self,
        target: &mut Pixmap,
        layer: &Layer,
        frame_cels: &[&Cel],
        _cels: &HashMap<Uuid, Cel>,
        tilesets: &HashMap<Uuid, Tileset>,
        palette: &Palette,
        layer_opacity: u8,
    ) -> CoreResult<()> {
        let _blend_mode = to_tiny_skia_blend_mode(layer.blend_mode);

        for cel in frame_cels {
            if cel.layer_uuid != layer.uuid || cel.cel_type != CelType::Tilemap {
                continue;
            }

            let tileset_uuid = layer.tileset_uuid.or(cel.tileset_uuid);
            let Some(tileset_uuid) = tileset_uuid else { continue };
            let Some(tileset) = tilesets.get(&tileset_uuid) else { continue };

            let tilemap_data = cel.user_data.get_property("tilemap")
                .or_else(|| cel.user_data.get_property("tilemap_data"));

            let Some(tilemap) = tilemap_data else { continue };
            let Some(tile_bytes) = tilemap.as_bytes() else { continue };

            // Get tileset image from cel or layer user data
            let tileset_image = cel.user_data.get_property("tileset_image")
                .or_else(|| layer.user_data.get_property("tileset_image"));

            if let Some(image_data) = tileset_image {
                if let Some(image_bytes) = image_data.as_bytes() {
                    let tileset_pixmap = self.create_tileset_pixmap(
                        &image_bytes,
                        (tileset.tile_width as u32) * (tileset.tile_count as u32),
                        tileset.tile_height as u32,
                        palette,
                    )?;

                    super::tileset_renderer::render_tilemap(
                        target,
                        &tileset_pixmap,
                        &tile_bytes,
                        tileset,
                        cel.position,
                        layer.blend_mode,
                        ((cel.opacity as u16 * layer_opacity as u16) / 255) as u8,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Create a pixmap from cel pixel data
    fn create_cel_pixmap(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        palette: &Palette,
        cel_opacity: u8,
    ) -> CoreResult<Pixmap> {
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| CoreError::Internal("Failed to create cel pixmap".into()))?;

        let _bytes_per_pixel = match palette.colors.first().map(|_| 1).unwrap_or(4) {
            // We can't determine color mode from palette alone, default to RGBA
            // In practice, the color mode should be passed from sprite metadata
            1 => 1, // Indexed
            _ => 4, // Default to RGBA
        };

        // We need the color mode to properly decode - default to RGBA for now
        let color_mode = ColorMode::Rgba;
        let expected_len = (width * height * color_mode.bytes_per_pixel() as u32) as usize;
        if pixels.len() < expected_len {
            return Err(CoreError::PixelDataSizeMismatch { expected: expected_len, actual: pixels.len() });
        }

        let opacity_factor = cel_opacity as f32 / 255.0;

        match color_mode {
            ColorMode::Indexed => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let palette_idx = pixels[idx] as usize;
                        let color = palette.colors.get(palette_idx)
                            .cloned()
                            .map(|c| c.to_rgba())
                            .unwrap_or(Rgba::TRANSPARENT);
                        let alpha = (color.a as f32 * opacity_factor) as u8;
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(color.r, color.g, color.b, alpha).unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::Grayscale => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx];
                        let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                        let alpha = (alpha as f32 * opacity_factor) as u8;
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::Rgba => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let r = pixels[idx];
                        let g = pixels.get(idx + 1).copied().unwrap_or(0);
                        let b = pixels.get(idx + 2).copied().unwrap_or(0);
                        let a = pixels.get(idx + 3).copied().unwrap_or(255);
                        let alpha = (a as f32 * opacity_factor) as u8;
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(r, g, b, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::GrayscaleAlpha => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx];
                        let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                        let alpha = (alpha as f32 * opacity_factor) as u8;
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
        }

        Ok(pixmap)
    }

    /// Create a pixmap from tileset image data
    fn create_tileset_pixmap(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        palette: &Palette,
    ) -> CoreResult<Pixmap> {
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| CoreError::Internal("Failed to create tileset pixmap".into()))?;

        // Default to RGBA for tileset images
        let color_mode = ColorMode::Rgba;
        let bytes_per_pixel = color_mode.bytes_per_pixel() as u32;
        let expected_len = (width * height * bytes_per_pixel) as usize;
        if pixels.len() < expected_len {
            return Err(CoreError::PixelDataSizeMismatch { expected: expected_len, actual: pixels.len() });
        }

        match color_mode {
            ColorMode::Indexed => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let palette_idx = pixels[idx] as usize;
                        let color = palette.colors.get(palette_idx)
                            .cloned()
                            .map(|c| c.to_rgba())
                            .unwrap_or(oversprite_core::Rgba::TRANSPARENT);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(color.r, color.g, color.b, color.a)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::Grayscale => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx];
                        let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::Rgba => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let r = pixels[idx];
                        let g = pixels.get(idx + 1).copied().unwrap_or(0);
                        let b = pixels.get(idx + 2).copied().unwrap_or(0);
                        let a = pixels.get(idx + 3).copied().unwrap_or(255);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(r, g, b, a)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::GrayscaleAlpha => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx];
                        let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
        }

        Ok(pixmap)
    }

    /// Create a pixmap from raw bytes (for onion skin)
    pub fn create_image_from_bytes(
        pixels: &[u8],
        width: u32,
        height: u32,
        color_mode: ColorMode,
    ) -> CoreResult<Pixmap> {
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| CoreError::Internal("Failed to create pixmap".into()))?;

        let bytes_per_pixel = color_mode.bytes_per_pixel() as u32;
        let expected_len = (width * height * bytes_per_pixel) as usize;
        if pixels.len() < expected_len {
            return Err(CoreError::PixelDataSizeMismatch { expected: expected_len, actual: pixels.len() });
        }

        match color_mode {
            ColorMode::Indexed => {
                // For indexed, we need a palette - use grayscale as fallback
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let gray = pixels[idx];
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, 255)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::Grayscale => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx];
                        let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::Rgba => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let r = pixels[idx];
                        let g = pixels.get(idx + 1).copied().unwrap_or(0);
                        let b = pixels.get(idx + 2).copied().unwrap_or(0);
                        let a = pixels.get(idx + 3).copied().unwrap_or(255);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(r, g, b, a)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
            ColorMode::GrayscaleAlpha => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 2) as usize;
                        let gray = pixels[idx];
                        let alpha = pixels.get(idx + 1).copied().unwrap_or(255);
                        let pixel_idx = (y * width + x) as usize;
                        if let Some(pixels_mut) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
        }

        Ok(pixmap)
    }

    /// Render all layers for a frame
    pub fn render_all_layers(
        &self,
        canvas: &mut crate::canvas::Canvas2D,
        sprite: &Sprite,
        frame_index: usize,
    ) -> CoreResult<()> {
        if let Some(frame_uuid) = sprite.frames.order.get(frame_index) {
            if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                for layer_uuid in &sprite.layers.order {
                    if let Some(layer) = sprite.layer_data.get(layer_uuid) {
                        if layer.visible {
                            let frame_cels: Vec<&Cel> = frame.cels.iter()
                                .filter_map(|c| sprite.cel_data.get(c))
                                .filter(|c| c.layer_uuid == layer.uuid)
                                .collect();

                            self.render_layer(
                                canvas.pixmap,
                                layer,
                                &frame_cels,
                                &sprite.cel_data,
                                &sprite.tilesets,
                                &sprite.global_palette,
                                None,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Draw layer bounds for debugging
    fn draw_layer_bounds(&self, _target: &mut Pixmap, _layer: &Layer) -> CoreResult<()> {
        // This is a simplified debug drawing - would need a proper debug overlay system
        Ok(())
    }

    /// Render multiple layers in order (bottom to top)
    pub fn render_layers(
        &self,
        target: &mut Pixmap,
        layer_order: &[Uuid],
        layers: &HashMap<Uuid, Layer>,
        frame_cels: &[&Cel],
        cels: &HashMap<Uuid, Cel>,
        tilesets: &HashMap<Uuid, Tileset>,
        palette: &Palette,
    ) -> CoreResult<()> {
        for layer_uuid in layer_order {
            if let Some(layer) = layers.get(layer_uuid) {
                self.render_layer(target, layer, frame_cels, cels, tilesets, palette, None)?;
            }
        }
        Ok(())
    }
}

/// Render a single cel to a target pixmap
pub fn render_cel_to_pixmap(
    target: &mut Pixmap,
    cel: &Cel,
    palette: &Palette,
    position: Point,
    blend_mode: BlendMode,
    opacity: u8,
) -> CoreResult<()> {
    let pixels = cel.user_data.get_property("pixels")
        .or_else(|| cel.user_data.get_property("raw_pixels"))
        .or_else(|| cel.user_data.get_property("compressed_pixels"));

    let Some(pixels) = pixels else { return Ok(()) };
    let Some(pixel_bytes) = pixels.as_bytes() else { return Ok(()) };

    let width = cel.bounds.w;
    let height = cel.bounds.h;

    if width == 0 || height == 0 {
        return Ok(());
    }

    let mut cel_pixmap = Pixmap::new(width, height)
        .ok_or_else(|| CoreError::Internal("Failed to create cel pixmap".into()))?;

    // Default to RGBA
    let color_mode = ColorMode::Rgba;
    let bytes_per_pixel = color_mode.bytes_per_pixel() as u32;

    let expected_len = (width * height * bytes_per_pixel) as usize;
    if pixel_bytes.len() < expected_len {
        return Err(CoreError::PixelDataSizeMismatch { expected: expected_len, actual: pixel_bytes.len() });
    }

    let opacity_factor = opacity as f32 / 255.0;

    match color_mode {
        ColorMode::Indexed => {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    let palette_idx = pixel_bytes[idx] as usize;
                    let color = palette.colors.get(palette_idx)
                        .cloned()
                        .map(|c| c.to_rgba())
                        .unwrap_or(Rgba::TRANSPARENT);
                    let alpha = (color.a as f32 * opacity_factor) as u8;
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = cel_pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(color.r, color.g, color.b, alpha).unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
        ColorMode::Rgba => {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    let r = pixel_bytes[idx];
                    let g = pixel_bytes.get(idx + 1).copied().unwrap_or(0);
                    let b = pixel_bytes.get(idx + 2).copied().unwrap_or(0);
                    let a = pixel_bytes.get(idx + 3).copied().unwrap_or(255);
                    let alpha = (a as f32 * opacity_factor) as u8;
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = cel_pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(r, g, b, alpha)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
        _ => {
            // Grayscale and GrayscaleAlpha handled similarly
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * bytes_per_pixel) as usize;
                    let gray = pixel_bytes[idx];
                    let alpha = if bytes_per_pixel > 1 {
                        pixel_bytes.get(idx + 1).copied().unwrap_or(255)
                    } else {
                        255
                    };
                    let alpha = (alpha as f32 * opacity_factor) as u8;
                    let pixel_idx = (y * width + x) as usize;
                    if let Some(pixels_mut) = cel_pixmap.pixels_mut().get_mut(pixel_idx) {
                        *pixels_mut = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                    }
                }
            }
        }
    }

    let paint = PixmapPaint {
        blend_mode: to_tiny_skia_blend_mode(blend_mode),
        opacity: opacity_factor,
        ..Default::default()
    };

    let transform = Transform::from_translate(position.x as f32, position.y as f32);
    target.draw_pixmap(0, 0, cel_pixmap.as_ref(), &paint, transform, None);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_renderer_creation() {
        let renderer = LayerRenderer::new();
        assert!(!renderer.debug_bounds);
        assert!(!renderer.debug_names);
    }
}