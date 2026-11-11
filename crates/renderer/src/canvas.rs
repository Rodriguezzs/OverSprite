//! OverSprite Renderer - Canvas Module
//!
//! High-level canvas abstraction for drawing operations using tiny-skia.

use oversprite_core::{CoreError, CoreResult, Rgba, Size, Rect};
use tiny_skia::{Color, Paint, Path, PathBuilder, Rect as TinyRect, Transform, Pixmap, PixmapPaint, Stroke, FillRule};

/// Canvas for drawing operations
pub struct Canvas2D<'a> {
    pub pixmap: &'a mut Pixmap,
    pub size: Size,
    pub scale: f32,
    pub background: Rgba,
}

impl<'a> Canvas2D<'a> {
    pub fn new(pixmap: &'a mut Pixmap, size: Size, scale: f32) -> CoreResult<Self> {
        Ok(Self {
            pixmap,
            size,
            scale,
            background: Rgba::TRANSPARENT,
        })
    }

    pub fn set_background(&mut self, color: Rgba) {
        self.background = color;
    }

    pub fn clear(&mut self) {
        let color = Color::from_rgba8(self.background.r, self.background.g, self.background.b, self.background.a);
        self.pixmap.fill(color);
    }

    pub fn clear_with(&mut self, color: Rgba) {
        let skia_color = Color::from_rgba8(color.r, color.g, color.b, color.a);
        self.pixmap.fill(skia_color);
    }

    pub fn draw_pixmap(&mut self, pixmap: &Pixmap, x: f32, y: f32, paint: Option<&PixmapPaint>) {
        let transform = Transform::from_translate(x, y);
        if let Some(paint) = paint {
            self.pixmap.draw_pixmap(0, 0, pixmap.as_ref(), paint, transform, None);
        } else {
            let default_paint = PixmapPaint::default();
            self.pixmap.draw_pixmap(0, 0, pixmap.as_ref(), &default_paint, transform, None);
        }
    }

    pub fn draw_pixmap_rect(&mut self, pixmap: &Pixmap, src: Rect, dst: Rect, paint: Option<&PixmapPaint>) {
        let src_rect = TinyRect::from_xywh(src.x as f32, src.y as f32, src.w as f32, src.h as f32);
        let dst_rect = TinyRect::from_xywh(dst.x as f32, dst.y as f32, dst.w as f32, dst.h as f32);

        if let (Some(src_rect), Some(dst_rect)) = (src_rect, dst_rect) {
            let transform = Transform::from_translate(dst_rect.x(), dst_rect.y());
            let scale_x = dst_rect.width() / src_rect.width();
            let scale_y = dst_rect.height() / src_rect.height();
            let transform = transform.post_scale(scale_x, scale_y);
            let transform = transform.pre_translate(-src_rect.x(), -src_rect.y());

            if let Some(paint) = paint {
                self.pixmap.draw_pixmap(0, 0, pixmap.as_ref(), paint, transform, None);
            } else {
                let default_paint = PixmapPaint::default();
                self.pixmap.draw_pixmap(0, 0, pixmap.as_ref(), &default_paint, transform, None);
            }
        }
    }

    pub fn draw_rect(&mut self, rect: Rect, paint: &Paint) {
        if let Some(tiny_rect) = TinyRect::from_xywh(rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32) {
            self.pixmap.fill_rect(tiny_rect, paint, Transform::identity(), None);
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Rgba) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, color.a));
        paint.anti_alias = true;
        self.draw_rect(rect, &paint);
    }

    pub fn stroke_rect(&mut self, rect: Rect, color: Rgba, width: f32) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, color.a));
        paint.anti_alias = true;

        if let Some(tiny_rect) = TinyRect::from_xywh(rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32) {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_rect.x(), tiny_rect.y(), tiny_rect.width(), tiny_rect.height());
            if let Some(path) = pb.finish() {
                let stroke = Stroke {
                    width,
                    ..Default::default()
                };
                self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Rgba, width: f32) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, color.a));
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.move_to(x1, y1);
        pb.line_to(x2, y2);
        if let Some(path) = pb.finish() {
            let stroke = Stroke {
                width,
                ..Default::default()
            };
            self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    pub fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgba, filled: bool) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, color.a));
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        if let Some(path) = pb.finish() {
            if filled {
                self.pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            } else {
                let stroke = Stroke {
                    width: 1.0,
                    ..Default::default()
                };
                self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }

    pub fn draw_path(&mut self, path: &Path, paint: &Paint, filled: bool) {
        if filled {
            self.pixmap.fill_path(path, paint, FillRule::Winding, Transform::identity(), None);
        } else {
            let stroke = Stroke::default();
            self.pixmap.stroke_path(path, paint, &stroke, Transform::identity(), None);
        }
    }

    pub fn save(&mut self) {
        // tiny-skia doesn't have explicit save/restore, we'd need to manage transform stack manually
        // For now, we'll just note that this is a no-op in tiny-skia
    }

    pub fn restore(&mut self) {
        // No-op in tiny-skia
    }

    pub fn translate(&mut self, _dx: f32, _dy: f32) {
        // Transform management would need a stack - simplified for now
    }

    pub fn scale(&mut self, _sx: f32, _sy: f32) {
        // Transform management would need a stack - simplified for now
    }

    pub fn rotate(&mut self, _degrees: f32) {
        // Transform management would need a stack - simplified for now
    }

    pub fn clip_rect(&mut self, _rect: Rect) {
        // Clipping not directly supported in tiny-skia's simple API
        // Would need to use a mask or path-based clipping
    }

    pub fn reset_clip(&mut self) {
        // No-op
    }

    /// Draw text (placeholder - would need a font system)
    pub fn draw_text(&mut self, _text: &str, _x: f32, _y: f32, _color: Rgba, _size: f32) {
        // Text rendering requires additional dependencies (e.g., rusttype, ab_glyph)
        // Placeholder for future implementation
    }

    /// Draw an ellipse
    pub fn draw_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgba, filled: bool) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, color.a));
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        // Approximate ellipse with bezier curves
        let kappa = 0.5522848; // 4/3 * (sqrt(2) - 1)
        let ox = rx * kappa;
        let oy = ry * kappa;

        pb.move_to(cx - rx, cy);
        pb.cubic_to(cx - rx, cy - oy, cx - ox, cy - ry, cx, cy - ry);
        pb.cubic_to(cx + ox, cy - ry, cx + rx, cy - oy, cx + rx, cy);
        pb.cubic_to(cx + rx, cy + oy, cx + ox, cy + ry, cx, cy + ry);
        pb.cubic_to(cx - ox, cy + ry, cx - rx, cy + oy, cx - rx, cy);
        pb.close();

        if let Some(path) = pb.finish() {
            if filled {
                self.pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            } else {
                let stroke = Stroke::default();
                self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }
}

/// Offscreen canvas for rendering to texture
pub struct OffscreenCanvas {
    pixmap: Pixmap,
    size: Size,
}

impl OffscreenCanvas {
    pub fn new(size: Size) -> CoreResult<Self> {
        let pixmap = Pixmap::new(size.w, size.h)
            .ok_or_else(|| CoreError::Internal("Failed to create offscreen pixmap".into()))?;
        Ok(Self { pixmap, size })
    }

    pub fn canvas(&mut self) -> Canvas2D<'_> {
        Canvas2D::new(&mut self.pixmap, self.size, 1.0).unwrap()
    }

    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    pub fn pixmap_mut(&mut self) -> &mut Pixmap {
        &mut self.pixmap
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

/// Checkerboard pattern for transparency visualization
pub struct Checkerboard {
    tile_size: f32,
    light_color: Rgba,
    dark_color: Rgba,
}

impl Default for Checkerboard {
    fn default() -> Self {
        Self {
            tile_size: 16.0,
            light_color: Rgba::new(200, 200, 200, 255),
            dark_color: Rgba::new(160, 160, 160, 255),
        }
    }
}

impl Checkerboard {
    pub fn new(tile_size: f32) -> Self {
        Self { tile_size, ..Default::default() }
    }

    pub fn with_colors(light: Rgba, dark: Rgba) -> Self {
        Self { light_color: light, dark_color: dark, ..Default::default() }
    }

    pub fn draw(&self, canvas: &mut Canvas2D, bounds: Rect) {
        let start_x = (bounds.x as f32 / self.tile_size).floor() as i32;
        let start_y = (bounds.y as f32 / self.tile_size).floor() as i32;
        let end_x = ((bounds.x + bounds.w as i32) as f32 / self.tile_size).ceil() as i32;
        let end_y = ((bounds.y + bounds.h as i32) as f32 / self.tile_size).ceil() as i32;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let tile_rect = Rect::new(
                    (x as f32 * self.tile_size) as i32,
                    (y as f32 * self.tile_size) as i32,
                    self.tile_size as u32,
                    self.tile_size as u32,
                );
                let color = if (x + y) % 2 == 0 { self.light_color } else { self.dark_color };
                canvas.fill_rect(tile_rect, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkerboard() {
        let cb = Checkerboard::default();
        assert_eq!(cb.tile_size, 16.0);
    }

    #[test]
    fn test_offscreen_canvas() {
        let canvas = OffscreenCanvas::new(Size::new(100, 100));
        assert!(canvas.is_ok());
    }
}