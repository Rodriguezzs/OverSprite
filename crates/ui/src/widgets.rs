//! OverSprite UI - Widgets Module
//!
//! Custom widgets for the pixel art editor.

use eframe::egui;
use oversprite_core::{Rgba, Point, Palette, PaletteColor};
use egui::{Color32, Stroke, Vec2, Rect as EguiRect, Pos2, Sense, Response, Ui, Painter};

/// Color swatch widget for palette display
pub fn color_swatch(ui: &mut Ui, color: Rgba, size: f32, selected: bool) -> Response {
    let c = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, c);
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0_f32, Color32::GRAY));

    if selected {
        painter.rect_stroke(rect, 2.0, Stroke::new(2.0_f32, Color32::WHITE));
    }

    response
}

/// Palette grid widget
pub fn palette_grid(ui: &mut Ui, palette: &Palette, cols: usize, size: f32, mut on_select: impl FnMut(usize, &PaletteColor)) {
    egui::Grid::new("palette_grid_widget")
        .num_columns(cols)
        .spacing([4.0, 4.0])
        .show(ui, |ui| {
            for (i, color) in palette.colors.iter().enumerate() {
                let response = color_swatch(ui, color.into(), size, false);
                if response.clicked() {
                    on_select(i, color);
                }
                if i % cols == cols - 1 { ui.end_row(); }
            }
        });
}

/// Tool button widget
pub fn tool_button(ui: &mut Ui, icon: &str, label: &str, selected: bool, shortcut: Option<&str>) -> Response {
    let text = if let Some(s) = shortcut {
        format!("{} {} ({})", icon, label, s)
    } else {
        format!("{} {}", icon, label)
    };

    ui.selectable_label(selected, text)
}

/// Layer list item widget
pub fn layer_list_item(
    ui: &mut Ui,
    name: &str,
    layer_type: &str,
    visible: bool,
    locked: bool,
    opacity: u8,
    blend_mode: &str,
    selected: bool,
) -> Response {
    let response = ui.horizontal(|ui| {
        // Visibility
        let mut vis = visible;
        ui.checkbox(&mut vis, "");

        // Lock
        let mut lock = locked;
        ui.checkbox(&mut lock, "");

        // Type icon
        let icon = match layer_type {
            "Image" => "🖼",
            "Group" => "📁",
            "Tilemap" => "🗺",
            _ => "❓",
        };
        ui.label(icon);

        // Name
        let name_response = ui.selectable_label(selected, name);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.small(format!("{}% {}", opacity, blend_mode));
        });

        name_response
    });

    response.response
}

/// Frame thumbnail widget for timeline
pub fn frame_thumbnail(
    ui: &mut Ui,
    frame_index: usize,
    duration_ms: u32,
    selected: bool,
    has_content: bool,
) -> Response {
    let size = Vec2::new(40.0, 30.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let painter = ui.painter();

    if has_content {
        painter.rect_filled(rect, 2.0, if selected {
            Color32::from_rgb(0, 128, 255)
        } else {
            Color32::from_rgb(80, 80, 80)
        });
    } else {
        painter.rect_stroke(rect, 2.0, Stroke::new(1.0_f32, Color32::GRAY));
    }

    if selected {
        painter.rect_stroke(rect, 2.0, Stroke::new(2.0_f32, Color32::WHITE));
    }

    // Frame number
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{}", frame_index + 1),
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );

    // Duration
    painter.text(
        rect.right_bottom() - Vec2::new(2.0, 2.0),
        egui::Align2::RIGHT_BOTTOM,
        format!("{}ms", duration_ms),
        egui::FontId::proportional(8.0),
        Color32::LIGHT_GRAY,
    );

    response
}

/// Canvas widget with zoom/pan support
pub struct CanvasWidget {
    pub zoom: f32,
    pub pan: Vec2,
    pub show_grid: bool,
    pub grid_size: f32,
    pub grid_color: Color32,
}

impl Default for CanvasWidget {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            show_grid: true,
            grid_size: 16.0,
            grid_color: Color32::from_rgb(200, 200, 200),
        }
    }
}

impl CanvasWidget {
    pub fn show(&mut self, ui: &mut Ui, content: impl FnOnce(&mut Ui, &Painter, f32, Vec2)) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

        // Handle pan
        if response.dragged() && ui.input(|i| i.modifiers.alt) {
            self.pan += response.drag_delta();
        }

        // Handle zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = (1.0 + scroll * 0.01).clamp(0.1, 64.0);
                self.zoom = (self.zoom * zoom_factor).clamp(0.1, 64.0);
            }
        }

        let painter = ui.painter_at(rect);

        // Draw checkerboard background
        self.draw_checkerboard(&painter, rect);

        // Draw grid
        if self.show_grid {
            self.draw_grid(&painter, rect);
        }

        // Call content with transform
        let _transform = egui::emath::TSTransform::from_translation(self.pan) * egui::emath::TSTransform::from_scaling(self.zoom);
        content(ui, &painter, self.zoom, self.pan);

        response
    }

    fn draw_checkerboard(&self, painter: &Painter, rect: EguiRect) {
        let tile_size = 16.0;
        let light = Color32::from_gray(200);
        let dark = Color32::from_gray(160);

        let start_x = (rect.left() / tile_size).floor() as i32;
        let start_y = (rect.top() / tile_size).floor() as i32;
        let end_x = (rect.right() / tile_size).ceil() as i32;
        let end_y = (rect.bottom() / tile_size).ceil() as i32;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let tile_rect = EguiRect::from_min_max(
                    Pos2::new(x as f32 * tile_size, y as f32 * tile_size),
                    Pos2::new((x + 1) as f32 * tile_size, (y + 1) as f32 * tile_size),
                );
                let color = if (x + y) % 2 == 0 { light } else { dark };
                painter.rect_filled(tile_rect, 0.0, color);
            }
        }
    }

    fn draw_grid(&self, painter: &Painter, rect: EguiRect) {
        let grid_size = self.grid_size * self.zoom;
        if grid_size < 4.0 { return; }

        let start_x = (rect.left() / grid_size).floor() as i32;
        let start_y = (rect.top() / grid_size).floor() as i32;
        let end_x = (rect.right() / grid_size).ceil() as i32;
        let end_y = (rect.bottom() / grid_size).ceil() as i32;

        let stroke = Stroke::new(1.0 / self.zoom, self.grid_color.gamma_multiply(0.3));

        for x in start_x..=end_x {
            let px = x as f32 * grid_size;
            painter.line_segment([Pos2::new(px, rect.top()), Pos2::new(px, rect.bottom())], stroke);
        }

        for y in start_y..=end_y {
            let py = y as f32 * grid_size;
            painter.line_segment([Pos2::new(rect.left(), py), Pos2::new(rect.right(), py)], stroke);
        }
    }
}

/// Property grid widget
pub fn property_grid(ui: &mut Ui, id: &str, rows: impl FnOnce(&mut Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([8.0, 4.0])
        .striped(true)
        .show(ui, rows);
}

/// Animation playback controls
pub fn playback_controls(ui: &mut Ui, playing: &mut bool, looping: &mut bool, onion_skin: &mut bool, fps: &mut u32) {
    ui.horizontal(|ui| {
        if ui.button(if *playing { "⏸" } else { "▶" }).clicked() {
            *playing = !*playing;
        }
        if ui.button("⏹").clicked() {
            *playing = false;
        }
        ui.separator();
        if ui.button("⏮").clicked() {}
        if ui.button("⏪").clicked() {}
        if ui.button("⏩").clicked() {}
        if ui.button("⏭").clicked() {}

        ui.separator();
        ui.label("FPS:");
        ui.add(egui::DragValue::new(fps).range(1..=60).speed(1));
        ui.checkbox(looping, "🔁 Loop");
        ui.checkbox(onion_skin, "🧅 Onion");
    });
}

/// Status bar widget
pub fn status_bar(ui: &mut Ui, info: &str, cursor_pos: Option<Point>, zoom: f32, frame: usize, total_frames: usize) {
    ui.horizontal(|ui| {
        ui.label(info);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{}%", (zoom * 100.0) as u32));
            ui.separator();
            ui.label(format!("Frame {}/{}", frame + 1, total_frames));
            if let Some(pos) = cursor_pos {
                ui.separator();
                ui.label(format!("X:{} Y:{}", pos.x, pos.y));
            }
        });
    });
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
}