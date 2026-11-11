//! OverSprite Renderer
//!
//! High-performance software renderer using tiny-skia for pixel-perfect rendering.

pub mod blend;
pub mod canvas;
pub mod layer_renderer;
pub mod tileset_renderer;
pub mod onion_skin;
pub mod thumbnail;

use oversprite_core::{
    BlendMode, Cel, CelType, ColorMode, CoreError, CoreResult, Frame, Grid, Layer, LayerType,
    Palette, PaletteColor, Rgba, Size, Sprite, Tileset, Uuid,
};
use tiny_skia::{Pixmap, PixmapPaint, Transform, PremultipliedColorU8};
use std::collections::HashMap;
use rmp_serde;

/// Main renderer for OverSprite sprites
pub struct Renderer {
    /// Canvas for rendering
    canvas: Pixmap,
    /// Sprite dimensions
    size: Size,
    /// Scale factor for display
    scale: f32,
    /// Background color (checkerboard for transparency)
    background: Rgba,
    /// Whether to show checkerboard
    show_checkerboard: bool,
    /// Checkerboard tile size
    checkerboard_tile_size: f32,
}

impl Renderer {
    /// Create a new renderer for the given sprite size
    pub fn new(size: Size, scale: f32) -> CoreResult<Self> {
        let canvas = Pixmap::new(size.w, size.h)
            .ok_or_else(|| CoreError::Internal("Failed to create pixmap".into()))?;

        Ok(Self {
            canvas,
            size,
            scale,
            background: Rgba::TRANSPARENT,
            show_checkerboard: true,
            checkerboard_tile_size: 16.0,
        })
    }

    /// Create a renderer with custom background
    pub fn with_background(size: Size, scale: f32, background: Rgba) -> CoreResult<Self> {
        let mut renderer = Self::new(size, scale)?;
        renderer.background = background;
        renderer.show_checkerboard = background.a == 0;
        Ok(renderer)
    }

    /// Get the canvas size
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get the scale factor
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Set the scale factor
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.1);
    }

    /// Toggle checkerboard display
    pub fn set_checkerboard(&mut self, show: bool) {
        self.show_checkerboard = show;
    }

    /// Set checkerboard tile size
    pub fn set_checkerboard_tile_size(&mut self, size: f32) {
        self.checkerboard_tile_size = size.max(1.0);
    }

    /// Get the underlying pixmap
    pub fn pixmap(&self) -> &Pixmap {
        &self.canvas
    }

    /// Get mutable pixmap for direct drawing
    pub fn pixmap_mut(&mut self) -> &mut Pixmap {
        &mut self.canvas
    }

    /// Clear the canvas with background
    pub fn clear(&mut self) {
        self.draw_checkerboard();
    }

    /// Draw checkerboard pattern for transparency visualization
    fn draw_checkerboard(&mut self) {
        if !self.show_checkerboard || self.background.a > 0 {
            // Solid background
            let color = tiny_skia::Color::from_rgba8(
                self.background.r, self.background.g, self.background.b, self.background.a
            );
            self.canvas.fill(color);
            return;
        }

        let light = tiny_skia::Color::from_rgba8(200, 200, 200, 255);
        let dark = tiny_skia::Color::from_rgba8(160, 160, 160, 255);
        let tile_size = self.checkerboard_tile_size as i32;

        for y in (0..self.size.h as i32).step_by(tile_size as usize) {
            for x in (0..self.size.w as i32).step_by(tile_size as usize) {
                let color = if ((x / tile_size) + (y / tile_size)) % 2 == 0 {
                    light
                } else {
                    dark
                };

                let w = (tile_size as u32).min(self.size.w - x as u32);
                let h = (tile_size as u32).min(self.size.h - y as u32);

                if let Some(rect) = tiny_skia::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
                    let mut paint = tiny_skia::Paint::default();
                    paint.set_color(color);
                    self.canvas.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        }
    }

    /// Draw a semi-transparent grid over the rendered canvas.
    ///
    /// The grid follows the sprite's grid settings; a zero-sized cell is a
    /// no-op so sprites without a grid render unchanged.
    pub fn draw_grid(&mut self, grid: &Grid, color: Rgba) {
        if grid.cell_size.w == 0 || grid.cell_size.h == 0 {
            return;
        }

        let stroke_color = tiny_skia::Color::from_rgba8(color.r, color.g, color.b, color.a);
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(stroke_color);

        let width = self.size.w as f32;
        let height = self.size.h as f32;
        let cell_w = grid.cell_size.w as f32;
        let cell_h = grid.cell_size.h as f32;

        let mut x = grid.origin.x as f32;
        while x <= width {
            let px = x.round();
            if let Some(rect) = tiny_skia::Rect::from_xywh(px, 0.0, 1.0, height) {
                self.canvas.fill_rect(rect, &paint, Transform::identity(), None);
            }
            x += cell_w;
        }

        let mut y = grid.origin.y as f32;
        while y <= height {
            let py = y.round();
            if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, py, width, 1.0) {
                self.canvas.fill_rect(rect, &paint, Transform::identity(), None);
            }
            y += cell_h;
        }
    }

    /// Render a complete sprite to the canvas
    pub fn render_sprite(
        &mut self,
        sprite: &Sprite,
        frame_index: usize,
        layers: &HashMap<Uuid, Layer>,
        cels: &HashMap<Uuid, Cel>,
        tilesets: &HashMap<Uuid, Tileset>,
        palette: &Palette,
        onion_skin: Option<&OnionSkinConfig>,
    ) -> CoreResult<()> {
        self.clear();

        // Get the frame
        let frame = sprite.frame_data.get(&sprite.frames.order.get(frame_index).copied().unwrap_or_default())
            .ok_or_else(|| CoreError::FrameNotFound(frame_index.to_string()))?;

        // Render layers in order (bottom to top)
        for layer_uuid in &sprite.layers.order {
            if let Some(layer) = layers.get(layer_uuid) {
                if !layer.visible {
                    continue;
                }
                self.render_layer(layer, frame, cels, tilesets, palette, sprite, frame_index, onion_skin)?;
            }
        }

        Ok(())
    }

    /// Render a single layer
    fn render_layer(
        &mut self,
        layer: &Layer,
        frame: &Frame,
        cels: &HashMap<Uuid, Cel>,
        tilesets: &HashMap<Uuid, Tileset>,
        palette: &Palette,
        sprite: &Sprite,
        frame_index: usize,
        onion_skin: Option<&OnionSkinConfig>,
    ) -> CoreResult<()> {
        match layer.layer_type {
            LayerType::Image => {
                // Render all cels belonging to this layer in this frame
                for cel_uuid in &frame.cels {
                    if let Some(cel) = cels.get(cel_uuid) {
                        if cel.layer_uuid == layer.uuid {
                            self.render_cel(cel, layer, palette, tilesets, onion_skin, frame_index, sprite)?;
                        }
                    }
                }
            }
            LayerType::Tilemap => {
                // Render tilemap cels
                for cel_uuid in &frame.cels {
                    if let Some(cel) = cels.get(cel_uuid) {
                        if cel.layer_uuid == layer.uuid && cel.cel_type == CelType::Tilemap {
                            self.render_tilemap_cel(cel, layer, tilesets, sprite)?;
                        }
                    }
                }
            }
            LayerType::Group => {
                // Groups don't render directly, they just contain children
            }
        }
        Ok(())
    }

    /// Render a cel (image layer)
    fn render_cel(
        &mut self,
        cel: &Cel,
        layer: &Layer,
        palette: &Palette,
        _tilesets: &HashMap<Uuid, Tileset>,
        onion_skin: Option<&OnionSkinConfig>,
        _frame_index: usize,
        sprite: &Sprite,
    ) -> CoreResult<()> {
        // Get pixel data from cel user data
        let pixels = cel.user_data.get_property("pixels")
            .or_else(|| cel.user_data.get_property("raw_pixels"))
            .or_else(|| cel.user_data.get_property("compressed_pixels"));

        let Some(pixels) = pixels else { return Ok(()); };
        let Some(pixel_bytes) = pixels.as_bytes() else { return Ok(()); };

        let width = cel.bounds.w;
        let height = cel.bounds.h;

        if width == 0 || height == 0 {
            return Ok(());
        }

        // Create pixmap from pixel data
        let pixmap = self.create_pixmap_from_pixels(&pixel_bytes, width, height, palette, sprite.metadata.color_mode, layer.blend_mode, cel.opacity)?;

        // Apply onion skin if enabled
        if let Some(config) = onion_skin {
            if config.enabled {
                // Previous frame onion skin
                if config.show_previous {
                    // TODO: Implement previous frame rendering with opacity
                }
                // Next frame onion skin
                if config.show_next {
                    // TODO: Implement next frame rendering with opacity
                }
            }
        }

        // Draw the cel at its position
        let transform = Transform::from_translate(cel.position.x as f32, cel.position.y as f32);
        let paint = PixmapPaint {
            blend_mode: blend::to_tiny_skia_blend_mode(layer.blend_mode),
            opacity: (cel.opacity as f32 / 255.0) * (layer.opacity as f32 / 255.0),
            ..Default::default()
        };

        self.canvas.draw_pixmap(
            0, 0,
            pixmap.as_ref(),
            &paint,
            transform,
            None,
        );

        Ok(())
    }

    /// Render a tilemap cel
    fn render_tilemap_cel(
        &mut self,
        cel: &Cel,
        layer: &Layer,
        tilesets: &HashMap<Uuid, Tileset>,
        sprite: &Sprite,
    ) -> CoreResult<()> {
        let Some(tileset_uuid) = layer.tileset_uuid.or(cel.tileset_uuid) else { return Ok(()) };
        let Some(tileset) = tilesets.get(&tileset_uuid) else { return Ok(()) };

        let tilemap_data = cel.user_data.get_property("tilemap")
            .or_else(|| cel.user_data.get_property("tilemap_data"));

        let Some(tilemap) = tilemap_data else { return Ok(()); };
        let Some(tile_bytes) = tilemap.as_bytes() else { return Ok(()); };

        // Get tileset image if available
        let tileset_image = cel.user_data.get_property("tileset_image")
            .and_then(|v| v.as_bytes());

        if let Some(image_bytes) = tileset_image {
            let tile_pixmap = self.create_pixmap_from_pixels(
                &image_bytes,
                (tileset.tile_width as u32) * (tileset.tile_count as u32),
                tileset.tile_height as u32,
                &layer.user_data.get_property("palette")
                    .and_then(|v| v.as_bytes())
                    .as_ref()
                    .and_then(|b| rmp_serde::from_slice::<Palette>(b).ok())
                    .unwrap_or_else(|| Palette::default()),
                sprite.metadata.color_mode,
                layer.blend_mode,
                cel.opacity
            )?;

            tileset_renderer::render_tilemap(
                &mut self.canvas,
                &tile_pixmap,
                &tile_bytes,
                tileset,
                cel.position,
                layer.blend_mode,
                cel.opacity,
            )?;
        }

        Ok(())
    }

    /// Create a tiny-skia pixmap from raw pixel bytes
    fn create_pixmap_from_pixels(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        palette: &Palette,
        color_mode: ColorMode,
        _blend_mode: BlendMode,
        _opacity: u8,
    ) -> CoreResult<Pixmap> {
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| CoreError::Internal("Failed to create cel pixmap".into()))?;

        let bytes_per_pixel = match color_mode {
            ColorMode::Indexed => 1,
            ColorMode::Grayscale => 2,
            ColorMode::Rgba => 4,
            ColorMode::GrayscaleAlpha => 2,
        };

        let expected_len = (width * height * bytes_per_pixel) as usize;
        if pixels.len() < expected_len {
            return Err(CoreError::ValidationError("Pixel data too short".into()));
        }

        // Fill pixmap based on color mode
        match color_mode {
            ColorMode::Indexed => {
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let palette_idx = pixels[idx] as usize;
                        let color = palette.colors.get(palette_idx)
                            .cloned()
                            .unwrap_or_else(|| PaletteColor::default());
                        let pixel_idx = idx;
                        if let Some(pixel) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixel = PremultipliedColorU8::from_rgba(color.r, color.g, color.b, color.a)
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
                        if let Some(pixel) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixel = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
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
                        if let Some(pixel) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixel = PremultipliedColorU8::from_rgba(r, g, b, a)
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
                        if let Some(pixel) = pixmap.pixels_mut().get_mut(pixel_idx) {
                            *pixel = PremultipliedColorU8::from_rgba(gray, gray, gray, alpha)
                                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
                        }
                    }
                }
            }
        }

        Ok(pixmap)
    }

    /// Save the current canvas to a PNG file
    pub fn save_png(&self, path: &std::path::Path) -> CoreResult<()> {
        self.canvas.save_png(path)
            .map_err(|e| CoreError::IoError(std::io::Error::other(e.to_string())))
    }

    /// Get the rendered image as PNG bytes
    pub fn png_bytes(&self) -> CoreResult<Vec<u8>> {
        self.canvas.encode_png()
            .map_err(|e| CoreError::IoError(std::io::Error::other(e.to_string())))
    }
}

/// Onion skin configuration
#[derive(Debug, Clone)]
pub struct OnionSkinConfig {
    pub enabled: bool,
    pub show_previous: bool,
    pub show_next: bool,
    pub previous_opacity: f32,
    pub next_opacity: f32,
    pub previous_color: Rgba,
    pub next_color: Rgba,
    pub max_frames: usize,
}

impl Default for OnionSkinConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_previous: true,
            show_next: true,
            previous_opacity: 0.3,
            next_opacity: 0.3,
            previous_color: Rgba::new(255, 100, 100, 255), // Red tint
            next_color: Rgba::new(100, 100, 255, 255),     // Blue tint
            max_frames: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(Size::new(100, 100), 1.0);
        assert!(renderer.is_ok());
        let renderer = renderer.unwrap();
        assert_eq!(renderer.size(), Size::new(100, 100));
    }

    #[test]
    fn test_renderer_with_background() {
        let renderer = Renderer::with_background(Size::new(50, 50), 1.0, Rgba::WHITE);
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_checkerboard_toggle() {
        let mut renderer = Renderer::new(Size::new(32, 32), 1.0).unwrap();
        renderer.set_checkerboard(false);
        renderer.clear();
        // Should have solid transparent background
    }
}