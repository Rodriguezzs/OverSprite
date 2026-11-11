//! OverSprite UI
//!
//! Editor UI built with egui, providing the main application window,
//! canvas, panels, and all editor components.

pub mod app;
pub mod canvas;
pub mod dialogs;
pub mod shortcuts;
pub mod theme;
pub mod widgets;

use eframe::egui;
use oversprite_core::{CoreResult, Sprite, Uuid, Point, ColorMode};

/// Main application state
pub struct OverSpriteApp {
    pub sprite: Option<Sprite>,
    pub current_frame: usize,
    pub current_tool: Tool,
    pub zoom: f32,
    pub pan: Point,
    pub show_grid: bool,
    pub show_onion_skin: bool,
    pub onion_skin_frames: usize,
    pub playing: bool,          // Timeline playback
    pub play_loop: bool,        // Loop playback
    pub ui_state: UiState,
    pub renderer: Option<oversprite_renderer::Renderer>,
    pub recent_files: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Eraser,
    Eyedropper,
    Move,
    Zoom,
    Hand,
    Selection,
    PaintBucket,
    Line,
    Rectangle,
    Ellipse,
    Spray,
    Text,
}

#[derive(Debug, Default)]
pub struct UiState {
    // Dock panels - animated expand/collapse
    pub layers_dock_open: bool,
    pub layers_dock_collapsed: bool,      // Icon-only mode (48px)
    pub properties_dock_open: bool,
    pub properties_dock_collapsed: bool,  // Icon-only mode
    pub timeline_dock_open: bool,
    pub timeline_dock_collapsed: bool,    // Minimal height (60px)

    // Panel content state
    pub layers_panel_selected: Option<Uuid>,
    pub frames_panel_selected: Option<Uuid>,
    pub color_picker_open: bool,

    // Floating contextual toolbar (top-center, appears on tool selection)
    pub toolbar_visible: bool,
    pub toolbar_tool: Option<Tool>,

    // Viewport overlays
    pub show_rulers: bool,
    pub show_grid: bool,
    pub show_onion_skin: bool,
    pub onion_skin_frames: usize,
}

impl Default for OverSpriteApp {
    fn default() -> Self {
        Self {
            sprite: None,
            current_frame: 0,
            current_tool: Tool::Pencil,
            zoom: 1.0,
            pan: Point::zero(),
            show_grid: true,
            show_onion_skin: false,
            onion_skin_frames: 2,
            playing: false,
            play_loop: true,
            ui_state: UiState::default(),
            renderer: None,
            recent_files: Vec::new(),
        }
    }
}

impl OverSpriteApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_sprite(&mut self, width: u32, height: u32, color_mode: ColorMode) -> CoreResult<()> {
        self.sprite = Some(Sprite::new_with_size(width, height, color_mode));
        self.current_frame = 0;
        self.zoom = 1.0;
        self.pan = Point::zero();
        Ok(())
    }

    pub fn load_sprite(&mut self, path: &std::path::Path) -> CoreResult<()> {
        self.sprite = Some(oversprite_parser::read_sprite(path)?);
        self.current_frame = 0;
        Ok(())
    }

    pub fn save_sprite(&self, path: &std::path::Path) -> CoreResult<()> {
        if let Some(sprite) = &self.sprite {
            oversprite_parser::write_sprite(sprite, path)?;
        }
        Ok(())
    }
}

impl eframe::App for OverSpriteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Delegate to the modern layout in app.rs
        self.layout_ui(ctx);

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);
    }
}