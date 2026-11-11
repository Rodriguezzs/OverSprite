//! OverSprite UI - Canvas Module
//!
//! Canvas widget for the editor with zoom, pan, and grid support.

use eframe::egui;
use oversprite_core::Size;

/// Canvas widget for drawing and interaction
pub struct CanvasWidget {
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub show_grid: bool,
    pub grid_size: f32,
    pub grid_color: egui::Color32,
    pub background_color: egui::Color32,
}

impl Default for CanvasWidget {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            show_grid: true,
            grid_size: 16.0,
            grid_color: egui::Color32::from_gray(100),
            background_color: egui::Color32::from_gray(30),
        }
    }
}

impl CanvasWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui, &egui::Painter, f32, egui::Vec2)) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        // Handle pan with middle mouse or alt+drag
        if response.dragged() && (ui.input(|i| i.pointer.middle_down()) || ui.input(|i| i.modifiers.alt)) {
            self.pan += response.drag_delta();
        }

        // Handle zoom with scroll
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = (1.0 + scroll * 0.01).clamp(0.1, 64.0);
                self.zoom = (self.zoom * zoom_factor).clamp(0.1, 64.0);
            }
        }

        let painter = ui.painter_at(rect);

        // Draw background
        painter.rect_filled(rect, 0.0, self.background_color);

        // Draw checkerboard for transparency
        self.draw_checkerboard(&painter, rect);

        // Draw grid
        if self.show_grid {
            self.draw_grid(&painter, rect);
        }

        // Call content with transform info
        content(ui, &painter, self.zoom, self.pan);

        response
    }

    fn draw_checkerboard(&self, painter: &egui::Painter, rect: egui::Rect) {
        let tile_size = 16.0;
        let light = egui::Color32::from_gray(200);
        let dark = egui::Color32::from_gray(160);

        let start_x = (rect.left() / tile_size).floor() as i32;
        let start_y = (rect.top() / tile_size).floor() as i32;
        let end_x = (rect.right() / tile_size).ceil() as i32;
        let end_y = (rect.bottom() / tile_size).ceil() as i32;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let tile_rect = egui::Rect::from_min_max(
                    egui::pos2(x as f32 * tile_size, y as f32 * tile_size),
                    egui::pos2((x + 1) as f32 * tile_size, (y + 1) as f32 * tile_size),
                );
                let color = if (x + y) % 2 == 0 { light } else { dark };
                painter.rect_filled(tile_rect, 0.0, color);
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_size = self.grid_size * self.zoom;
        if grid_size < 4.0 { return; }

        let start_x = (rect.left() / grid_size).floor() as i32;
        let start_y = (rect.top() / grid_size).floor() as i32;
        let end_x = (rect.right() / grid_size).ceil() as i32;
        let end_y = (rect.bottom() / grid_size).ceil() as i32;

        let stroke = egui::Stroke::new(1.0 / self.zoom, self.grid_color.gamma_multiply(0.3));

        for x in start_x..=end_x {
            let px = x as f32 * grid_size;
            painter.line_segment([egui::pos2(px, rect.top()), egui::pos2(px, rect.bottom())], stroke);
        }

        for y in start_y..=end_y {
            let py = y as f32 * grid_size;
            painter.line_segment([egui::pos2(rect.left(), py), egui::pos2(rect.right(), py)], stroke);
        }
    }

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_pos: egui::Pos2, canvas_rect: egui::Rect) -> egui::Pos2 {
        let local_pos = screen_pos - canvas_rect.min;
        egui::pos2(
            (local_pos.x - self.pan.x) / self.zoom,
            (local_pos.y - self.pan.y) / self.zoom,
        )
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_pos: egui::Pos2, canvas_rect: egui::Rect) -> egui::Pos2 {
        egui::pos2(
            canvas_pos.x * self.zoom + self.pan.x + canvas_rect.min.x,
            canvas_pos.y * self.zoom + self.pan.y + canvas_rect.min.y,
        )
    }

    /// Get the visible canvas rect in canvas coordinates
    pub fn visible_rect(&self, canvas_rect: egui::Rect) -> egui::Rect {
        let min = self.screen_to_canvas(canvas_rect.min, canvas_rect);
        let max = self.screen_to_canvas(canvas_rect.max, canvas_rect);
        egui::Rect::from_two_pos(min, max)
    }
}

/// High-level canvas for rendering sprites
pub struct SpriteCanvas {
    pub widget: CanvasWidget,
    pub sprite_size: Size,
}

impl SpriteCanvas {
    pub fn new(sprite_size: Size) -> Self {
        Self {
            widget: CanvasWidget::new(),
            sprite_size,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, sprite: &oversprite_core::Sprite, frame_index: usize) -> egui::Response {
        self.widget.show(ui, |_ui, painter, zoom, pan| {
            // Render sprite using the renderer
            // This would integrate with oversprite_renderer
            Self::render_sprite_preview(painter, sprite, frame_index, zoom, pan);
        })
    }

    fn render_sprite_preview(painter: &egui::Painter, sprite: &oversprite_core::Sprite, frame_index: usize, zoom: f32, pan: egui::Vec2) {
        // Placeholder - would use actual renderer
        if let Some(frame_uuid) = sprite.frames.order.get(frame_index) {
            if let Some(_frame) = sprite.frame_data.get(frame_uuid) {
                let size = egui::vec2(
                    sprite.metadata.dimensions.w as f32 * zoom,
                    sprite.metadata.dimensions.h as f32 * zoom,
                );
                let rect = egui::Rect::from_min_size(
                    egui::pos2(pan.x, pan.y),
                    size,
                );
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0_f32, egui::Color32::RED));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_widget_default() {
        let canvas = CanvasWidget::default();
        assert_eq!(canvas.zoom, 1.0);
        assert!(canvas.show_grid);
    }

    #[test]
    fn test_sprite_canvas_creation() {
        let canvas = SpriteCanvas::new(Size::new(128, 128));
        assert_eq!(canvas.sprite_size.w, 128);
        assert_eq!(canvas.sprite_size.h, 128);
    }
}