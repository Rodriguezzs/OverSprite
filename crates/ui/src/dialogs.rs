//! OverSprite UI - Dialogs Module
//!
//! Modal dialogs for file operations, preferences, new sprite, etc.

use eframe::egui;
use oversprite_core::{ColorMode, Size};
use std::path::PathBuf;

/// Native file-dialog filters for the formats OverSprite opens and saves.
pub const OPEN_FILTERS: &[(&str, &[&str])] = &[
    ("OverSprite", &["ovs"]),
    ("Aseprite", &["aseprite", "ase"]),
];

/// Default extension used when the user has not typed one.
pub const DEFAULT_EXTENSION: &str = "ovs";

/// Append the default extension when the path has none.
pub fn with_default_extension(path: &PathBuf) -> PathBuf {
    if path.extension().is_some() {
        path.clone()
    } else {
        path.with_extension(DEFAULT_EXTENSION)
    }
}

/// Dialog state for the application
#[derive(Default)]
pub struct DialogState {
    pub show_new_sprite: bool,
    pub show_open_file: bool,
    pub show_save_as: bool,
    pub show_export: bool,
    pub show_preferences: bool,
    pub show_sprite_properties: bool,
    pub show_canvas_size: bool,
    pub show_about: bool,
    pub show_color_picker: bool,
    pub show_resize_image: bool,
}

impl DialogState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn close_all(&mut self) {
        self.show_new_sprite = false;
        self.show_open_file = false;
        self.show_save_as = false;
        self.show_export = false;
        self.show_preferences = false;
        self.show_sprite_properties = false;
        self.show_canvas_size = false;
        self.show_about = false;
        self.show_color_picker = false;
        self.show_resize_image = false;
    }
}

/// New sprite dialog
pub fn show_new_sprite_dialog(ctx: &egui::Context, state: &mut DialogState, on_create: impl FnOnce(Size, ColorMode)) {
    if !state.show_new_sprite { return; }

    let mut width = 128u32;
    let mut height = 128u32;
    let mut color_mode = ColorMode::Rgba;
    let mut frames = 1u32;
    let mut should_close = false;
    let mut create_action: Option<(Size, ColorMode)> = None;

    egui::Window::new("New Sprite")
        .resizable(false)
        .collapsible(false)
        .open(&mut state.show_new_sprite)
        .show(ctx, |ui| {
            egui::Grid::new("new_sprite_grid").num_columns(2).show(ui, |ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(&mut width).range(1..=8192));
                ui.end_row();

                ui.label("Height:");
                ui.add(egui::DragValue::new(&mut height).range(1..=8192));
                ui.end_row();

                ui.label("Color Mode:");
                egui::ComboBox::from_id_salt("new_color_mode")
                    .selected_text(format!("{:?}", color_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut color_mode, ColorMode::Rgba, "RGBA");
                        ui.selectable_value(&mut color_mode, ColorMode::Indexed, "Indexed");
                        ui.selectable_value(&mut color_mode, ColorMode::Grayscale, "Grayscale");
                        ui.selectable_value(&mut color_mode, ColorMode::GrayscaleAlpha, "Grayscale + Alpha");
                    });
                ui.end_row();

                ui.label("Frames:");
                ui.add(egui::DragValue::new(&mut frames).range(1..=1000));
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    create_action = Some((Size::new(width, height), color_mode));
                    should_close = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        state.show_new_sprite = false;
    }
    if let Some((size, mode)) = create_action {
        on_create(size, mode);
    }
}

/// File open dialog (uses native dialog)
pub fn show_open_file_dialog(state: &mut DialogState, _on_select: impl FnOnce(PathBuf)) {
    if !state.show_open_file { return; }

    // In a real app, this would use rfd::FileDialog
    // For now, just a placeholder
    state.show_open_file = false;
}

/// Save as dialog
pub fn show_save_as_dialog(ctx: &egui::Context, state: &mut DialogState, current_path: Option<&PathBuf>, on_save: impl FnOnce(PathBuf)) {
    if !state.show_save_as { return; }

    let mut path_str = current_path.map(|p| p.display().to_string()).unwrap_or_default();
    let mut should_close = false;
    let mut save_action: Option<PathBuf> = None;

    egui::Window::new("Save As")
        .resizable(false)
        .open(&mut state.show_save_as)
        .show(ctx, |ui| {
            ui.label("File path:");
            ui.text_edit_singleline(&mut path_str);
            ui.label(format!("Saved as OverSprite (.{DEFAULT_EXTENSION}) unless another extension is given."));

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    save_action = Some(with_default_extension(&PathBuf::from(path_str)));
                    should_close = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        state.show_save_as = false;
    }
    if let Some(path) = save_action {
        on_save(path);
    }
}

/// Export dialog
pub fn show_export_dialog(ctx: &egui::Context, state: &mut DialogState, on_export: impl FnOnce(PathBuf, ExportFormat)) {
    if !state.show_export { return; }

    let mut format = ExportFormat::Png;
    let mut path_str = String::new();
    let mut scale = 1.0f32;
    let mut should_close = false;
    let mut export_action: Option<(PathBuf, ExportFormat)> = None;

    egui::Window::new("Export")
        .resizable(false)
        .open(&mut state.show_export)
        .show(ctx, |ui| {
            egui::Grid::new("export_grid").num_columns(2).show(ui, |ui| {
                ui.label("Format:");
                egui::ComboBox::from_id_salt("export_format")
                    .selected_text(format!("{:?}", format))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut format, ExportFormat::Png, "PNG");
                        ui.selectable_value(&mut format, ExportFormat::Gif, "GIF");
                        ui.selectable_value(&mut format, ExportFormat::WebP, "WebP");
                        ui.selectable_value(&mut format, ExportFormat::Aseprite, "Aseprite");
                        ui.selectable_value(&mut format, ExportFormat::Ovs, "OverSprite (.ovs)");
                    });
                ui.end_row();

                ui.label("Scale:");
                ui.add(egui::DragValue::new(&mut scale).range(0.1..=10.0).speed(0.1));
                ui.end_row();

                ui.label("Output:");
                ui.text_edit_singleline(&mut path_str);
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Export").clicked() {
                    export_action = Some((PathBuf::from(path_str), format));
                    should_close = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        state.show_export = false;
    }
    if let Some((path, fmt)) = export_action {
        on_export(path, fmt);
    }
}

/// Preferences dialog
pub fn show_preferences_dialog(ctx: &egui::Context, state: &mut DialogState, prefs: &mut Preferences) {
    if !state.show_preferences { return; }

    let mut should_close = false;
    let mut reset_to_defaults = false;

    egui::Window::new("Preferences")
        .resizable(false)
        .min_width(400.0)
        .open(&mut state.show_preferences)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("General");
                ui.add_space(4.0);

                ui.checkbox(&mut prefs.show_grid, "Show grid by default");
                ui.checkbox(&mut prefs.show_onion_skin, "Enable onion skin by default");
                ui.checkbox(&mut prefs.auto_save, "Auto-save");
                ui.add(egui::DragValue::new(&mut prefs.auto_save_interval).range(1..=60).suffix(" min"));

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("Appearance");
                ui.add_space(4.0);

                egui::ComboBox::from_label("Theme")
                    .selected_text(format!("{:?}", prefs.theme))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut prefs.theme, Theme::Dark, "Dark");
                        ui.selectable_value(&mut prefs.theme, Theme::Light, "Light");
                        ui.selectable_value(&mut prefs.theme, Theme::System, "System");
                    });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("Shortcuts");
                ui.add_space(4.0);
                ui.label("Shortcut customization coming soon...");

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("Reset to Defaults").clicked() {
                        reset_to_defaults = true;
                    }
                    if ui.button("OK").clicked() {
                        should_close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });
        });

    if reset_to_defaults {
        *prefs = Preferences::default();
    }
    if should_close {
        state.show_preferences = false;
    }
}

/// Sprite properties dialog
pub fn show_sprite_properties_dialog(ctx: &egui::Context, state: &mut DialogState, sprite: &mut oversprite_core::Sprite) {
    if !state.show_sprite_properties { return; }

    let mut width = sprite.metadata.dimensions.w;
    let mut height = sprite.metadata.dimensions.h;
    let color_mode = sprite.metadata.color_mode;
    let mut fps = 12u32;
    let mut should_close = false;

    egui::Window::new("Sprite Properties")
        .resizable(false)
        .open(&mut state.show_sprite_properties)
        .show(ctx, |ui| {
            egui::Grid::new("sprite_props_grid").num_columns(2).show(ui, |ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(&mut width).range(1..=8192));
                ui.end_row();

                ui.label("Height:");
                ui.add(egui::DragValue::new(&mut height).range(1..=8192));
                ui.end_row();

                ui.label("Color Mode:");
                ui.label(format!("{:?}", color_mode));
                ui.end_row();

                ui.label("Frames:");
                ui.label(sprite.frames.order.len().to_string());
                ui.end_row();

                ui.label("Layers:");
                ui.label(sprite.layers.order.len().to_string());
                ui.end_row();

                ui.label("FPS:");
                ui.add(egui::DragValue::new(&mut fps).range(1..=60));
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    should_close = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        state.show_sprite_properties = false;
    }
}

/// Color picker dialog
pub fn show_color_picker_dialog(ctx: &egui::Context, state: &mut DialogState, color: &mut oversprite_core::Rgba) {
    if !state.show_color_picker { return; }

    let mut should_close = false;

    egui::Window::new("Color Picker")
        .resizable(false)
        .open(&mut state.show_color_picker)
        .show(ctx, |ui| {
            // HSV color wheel would go here
            // For now, simple RGBA sliders
            egui::Grid::new("color_picker_grid").num_columns(2).show(ui, |ui| {
                ui.label("R:");
                ui.add(egui::Slider::new(&mut color.r, 0..=255));
                ui.end_row();

                ui.label("G:");
                ui.add(egui::Slider::new(&mut color.g, 0..=255));
                ui.end_row();

                ui.label("B:");
                ui.add(egui::Slider::new(&mut color.b, 0..=255));
                ui.end_row();

                ui.label("A:");
                ui.add(egui::Slider::new(&mut color.a, 0..=255));
                ui.end_row();

                ui.label("Hex:");
                let mut hex = format!("{:02X}{:02X}{:02X}{:02X}", color.r, color.g, color.b, color.a);
                ui.text_edit_singleline(&mut hex);
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    should_close = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        state.show_color_picker = false;
    }
}

/// About dialog
pub fn show_about_dialog(ctx: &egui::Context, state: &mut DialogState) {
    if !state.show_about { return; }

    let mut should_close = false;

    egui::Window::new("About OverSprite")
        .resizable(false)
        .open(&mut state.show_about)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("OverSprite");
                ui.label("Version 1.0.0");
                ui.add_space(8.0);
                ui.label("A modern pixel art editor");
                ui.label("Compatible with Aseprite .aseprite files");
                ui.add_space(8.0);
                ui.hyperlink("https://oversprite.dev");
                ui.add_space(16.0);
                if ui.button("OK").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        state.show_about = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Gif,
    WebP,
    Aseprite,
    Ovs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone)]
pub struct Preferences {
    pub show_grid: bool,
    pub show_onion_skin: bool,
    pub auto_save: bool,
    pub auto_save_interval: u32,
    pub theme: Theme,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_onion_skin: false,
            auto_save: true,
            auto_save_interval: 5,
            theme: Theme::Dark,
        }
    }
}