//! OverSprite Renderer - Thumbnail Module
//!
//! Generates thumbnails for sprites at various sizes for file previews,
//! asset browsers, and UI components.

use oversprite_core::{CoreError, CoreResult, Sprite, Size, Rgba, Rect};
use crate::Renderer;
use tiny_skia::{Pixmap, Color};

/// Thumbnail generator
pub struct ThumbnailGenerator {
    pub sizes: Vec<u32>,
    pub background: Rgba,
    pub padding: f32,
    pub fit_mode: FitMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitMode {
    /// Fit within bounds, preserve aspect ratio
    Contain,
    /// Fill bounds, crop if needed
    Cover,
    /// Stretch to fill
    Fill,
}

impl Default for ThumbnailGenerator {
    fn default() -> Self {
        Self {
            sizes: vec![64, 128, 256, 512],
            background: Rgba::TRANSPARENT,
            padding: 0.0,
            fit_mode: FitMode::Contain,
        }
    }
}

impl ThumbnailGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sizes(mut self, sizes: Vec<u32>) -> Self {
        self.sizes = sizes;
        self
    }

    pub fn with_background(mut self, bg: Rgba) -> Self {
        self.background = bg;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_fit_mode(mut self, mode: FitMode) -> Self {
        self.fit_mode = mode;
        self
    }

    /// Generate thumbnails for all configured sizes
    pub fn generate(&self, sprite: &Sprite) -> CoreResult<Vec<(u32, Vec<u8>)>> {
        let mut thumbnails = Vec::new();

        for &size in &self.sizes {
            let data = self.generate_size(sprite, size)?;
            thumbnails.push((size, data));
        }

        Ok(thumbnails)
    }

    /// Generate thumbnail at specific size
    pub fn generate_size(&self, sprite: &Sprite, size: u32) -> CoreResult<Vec<u8>> {
        let canvas_size = self.calculate_canvas_size(sprite, size);

        // Create renderer
        let mut renderer = Renderer::new(canvas_size, 1.0)?;

        // Render first frame
        renderer.render_sprite(
            sprite,
            0,
            &sprite.layer_data,
            &sprite.cel_data,
            &sprite.tilesets,
            &sprite.global_palette,
            None,
        )?;

        // Get the rendered pixmap
        let rendered = renderer.pixmap().clone();

        // Create thumbnail pixmap
        let mut thumbnail = Pixmap::new(size, size)
            .ok_or_else(|| CoreError::Internal("Failed to create thumbnail pixmap".into()))?;

        // Fill with background
        let bg_color = Color::from_rgba8(
            self.background.r,
            self.background.g,
            self.background.b,
            self.background.a,
        );
        thumbnail.fill(bg_color);

        // Calculate draw rect based on fit mode
        let draw_rect = self.calculate_draw_rect(
            Rect::new(0, 0, size, size),
            Rect::new(0, 0, rendered.width(), rendered.height()),
        );

        // Draw the rendered image onto thumbnail with scaling
        let src_rect = tiny_skia::Rect::from_xywh(
            0.0, 0.0,
            rendered.width() as f32, rendered.height() as f32
        ).unwrap();

        let dst_rect = tiny_skia::Rect::from_xywh(
            draw_rect.x as f32,
            draw_rect.y as f32,
            draw_rect.w as f32,
            draw_rect.h as f32,
        ).unwrap();

        let scale_x = dst_rect.width() / src_rect.width();
        let scale_y = dst_rect.height() / src_rect.height();

        let transform = tiny_skia::Transform::from_translate(dst_rect.x(), dst_rect.y())
            .post_scale(scale_x, scale_y);

        let paint = tiny_skia::PixmapPaint::default();
        thumbnail.draw_pixmap(0, 0, rendered.as_ref(), &paint, transform, None);

        // Encode as PNG
        thumbnail.encode_png()
            .map_err(|e| CoreError::IoError(std::io::Error::other(e.to_string())))
    }

    fn calculate_canvas_size(&self, sprite: &Sprite, target_size: u32) -> Size {
        let sprite_w = sprite.metadata.dimensions.w as f32;
        let sprite_h = sprite.metadata.dimensions.h as f32;
        let aspect = sprite_w / sprite_h;

        match self.fit_mode {
            FitMode::Contain => {
                if aspect > 1.0 {
                    Size::new(target_size, (target_size as f32 / aspect) as u32)
                } else {
                    Size::new((target_size as f32 * aspect) as u32, target_size)
                }
            }
            FitMode::Cover => {
                if aspect > 1.0 {
                    Size::new((target_size as f32 * aspect) as u32, target_size)
                } else {
                    Size::new(target_size, (target_size as f32 / aspect) as u32)
                }
            }
            FitMode::Fill => Size::new(target_size, target_size),
        }
    }

    fn calculate_draw_rect(&self, target: Rect, source: Rect) -> Rect {
        let target_w = target.w as f32;
        let target_h = target.h as f32;
        let source_w = source.w as f32;
        let source_h = source.h as f32;

        let (draw_w, draw_h, draw_x, draw_y) = match self.fit_mode {
            FitMode::Contain => {
                let scale = (target_w / source_w).min(target_h / source_h);
                let w = source_w * scale;
                let h = source_h * scale;
                let x = target.x as f32 + (target_w - w) / 2.0;
                let y = target.y as f32 + (target_h - h) / 2.0;
                (w, h, x, y)
            }
            FitMode::Cover => {
                let scale = (target_w / source_w).max(target_h / source_h);
                let w = source_w * scale;
                let h = source_h * scale;
                let x = target.x as f32 + (target_w - w) / 2.0;
                let y = target.y as f32 + (target_h - h) / 2.0;
                (w, h, x, y)
            }
            FitMode::Fill => {
                (target_w, target_h, target.x as f32, target.y as f32)
            }
        };

        Rect::new(
            (draw_x + self.padding) as i32,
            (draw_y + self.padding) as i32,
            (draw_w - self.padding * 2.0).max(1.0) as u32,
            (draw_h - self.padding * 2.0).max(1.0) as u32,
        )
    }

    /// Generate animated thumbnail (placeholder - returns first frame as PNG)
    pub fn generate_animated(&self, sprite: &Sprite, size: u32, _fps: u32) -> CoreResult<Vec<u8>> {
        // For animated thumbnails, we'd need to render all frames
        // and encode as animated GIF/WebP - this is a placeholder
        self.generate_size(sprite, size)
    }

    /// Generate sprite sheet thumbnail (all frames in grid)
    pub fn generate_sprite_sheet(&self, sprite: &Sprite, size: u32, cols: u32) -> CoreResult<Vec<u8>> {
        let frame_count = sprite.frames.order.len() as u32;
        let rows = frame_count.div_ceil(cols);

        let cell_w = size / cols;
        let cell_h = size / rows;
        let sheet_size = Size::new(cell_w * cols, cell_h * rows);

        let mut renderer = Renderer::new(sheet_size, 1.0)?;

        // For now just return first frame - full implementation would render all frames
        renderer.render_sprite(
            sprite,
            0,
            &sprite.layer_data,
            &sprite.cel_data,
            &sprite.tilesets,
            &sprite.global_palette,
            None,
        )?;

        let _rendered = renderer.pixmap().clone();

        // Create sprite sheet pixmap
        let mut sheet = Pixmap::new(size, size)
            .ok_or_else(|| CoreError::Internal("Failed to create sprite sheet pixmap".into()))?;

        // Fill with background
        let bg_color = Color::from_rgba8(
            self.background.r,
            self.background.g,
            self.background.b,
            self.background.a,
        );
        sheet.fill(bg_color);

        // Render each frame into the grid
        for (i, _) in sprite.frames.order.iter().enumerate() {
            if i >= (cols * rows) as usize { break; }

            let col = (i as u32) % cols;
            let row = (i as u32) / cols;

            // Render this frame
            let mut frame_renderer = Renderer::new(Size::new(cell_w, cell_h), 1.0)?;
            frame_renderer.render_sprite(
                sprite,
                i,
                &sprite.layer_data,
                &sprite.cel_data,
                &sprite.tilesets,
                &sprite.global_palette,
                None,
            )?;

            let frame_image = frame_renderer.pixmap();

            let dst_x = col * cell_w;
            let dst_y = row * cell_h;

            let transform = tiny_skia::Transform::from_translate(dst_x as f32, dst_y as f32);
            let paint = tiny_skia::PixmapPaint::default();
            sheet.draw_pixmap(0, 0, frame_image.as_ref(), &paint, transform, None);
        }

        sheet.encode_png()
            .map_err(|e| CoreError::IoError(std::io::Error::other(e.to_string())))
    }
}

/// Quick thumbnail for UI previews (small, fast)
pub fn quick_thumbnail(sprite: &Sprite, max_size: u32) -> CoreResult<Vec<u8>> {
    let generator = ThumbnailGenerator::new()
        .with_sizes(vec![max_size])
        .with_background(Rgba::new(30, 30, 30, 255))
        .with_fit_mode(FitMode::Contain);

    let thumbnails = generator.generate(sprite)?;
    Ok(thumbnails.into_iter().next().map(|(_, data)| data).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_generator_creation() {
        let gen = ThumbnailGenerator::new();
        assert_eq!(gen.sizes, vec![64, 128, 256, 512]);
        assert_eq!(gen.fit_mode, FitMode::Contain);
    }

    #[test]
    fn test_calculate_canvas_size() {
        let gen = ThumbnailGenerator::new().with_fit_mode(FitMode::Contain);
        let sprite = Sprite::new_with_size(100, 50, oversprite_core::ColorMode::Rgba);
        let size = gen.calculate_canvas_size(&sprite, 128);
        // 100x50 = 2:1 aspect, so 128x64
        assert_eq!(size.w, 128);
        assert_eq!(size.h, 64);
    }

    #[test]
    fn test_fit_mode_cover() {
        let gen = ThumbnailGenerator::new().with_fit_mode(FitMode::Cover);
        let sprite = Sprite::new_with_size(100, 50, oversprite_core::ColorMode::Rgba);
        let size = gen.calculate_canvas_size(&sprite, 128);
        // Cover fills the target, so 256x128 (cropped)
        assert_eq!(size.w, 256);
        assert_eq!(size.h, 128);
    }
}