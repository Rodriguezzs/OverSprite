//! OverSprite UI - Shortcuts Module
//!
//! Keyboard shortcut handling for the editor.

use eframe::egui;
use crate::OverSpriteApp;

/// Shortcut definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shortcut {
    // File
    New,
    Open,
    Save,
    SaveAs,
    Export,
    Exit,

    // Edit
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    Deselect,

    // View
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    ToggleGrid,
    ToggleOnionSkin,
    ToggleFullscreen,

    // Sprite
    SpriteProperties,
    CanvasSize,
    ResizeImage,

    // Layer
    NewLayer,
    DeleteLayer,
    DuplicateLayer,
    MergeDown,
    GroupLayers,

    // Frame
    NewFrame,
    DeleteFrame,
    DuplicateFrame,
    PrevFrame,
    NextFrame,
    FirstFrame,
    LastFrame,

    // Tools
    ToolPencil,
    ToolEraser,
    ToolEyedropper,
    ToolMove,
    ToolZoom,
    ToolHand,
    ToolSelection,
    ToolPaintBucket,
    ToolLine,
    ToolRectangle,
    ToolEllipse,
    ToolSpray,
    ToolText,

    // Playback
    PlayPause,
    Stop,
    Loop,
    OnionSkinPrev,
    OnionSkinNext,

    // Palette
    ForegroundColor,
    BackgroundColor,
    SwapColors,
    DefaultColors,
}

impl Shortcut {
    pub fn default_key(&self) -> Option<(egui::Modifiers, egui::Key)> {
        use egui::{Modifiers, Key};
        Some(match self {
            // File
            Shortcut::New => (Modifiers::CTRL, Key::N),
            Shortcut::Open => (Modifiers::CTRL, Key::O),
            Shortcut::Save => (Modifiers::CTRL, Key::S),
            Shortcut::SaveAs => (Modifiers::CTRL | Modifiers::SHIFT, Key::S),
            Shortcut::Export => (Modifiers::CTRL | Modifiers::SHIFT, Key::E),
            Shortcut::Exit => (Modifiers::CTRL, Key::Q),

            // Edit
            Shortcut::Undo => (Modifiers::CTRL, Key::Z),
            Shortcut::Redo => (Modifiers::CTRL | Modifiers::SHIFT, Key::Z),
            Shortcut::Cut => (Modifiers::CTRL, Key::X),
            Shortcut::Copy => (Modifiers::CTRL, Key::C),
            Shortcut::Paste => (Modifiers::CTRL, Key::V),
            Shortcut::Delete => (Modifiers::NONE, Key::Delete),
            Shortcut::SelectAll => (Modifiers::CTRL, Key::A),
            Shortcut::Deselect => (Modifiers::CTRL, Key::D),

            // View
            Shortcut::ZoomIn => (Modifiers::CTRL, Key::Equals),
            Shortcut::ZoomOut => (Modifiers::CTRL, Key::Minus),
            Shortcut::ZoomReset => (Modifiers::CTRL, Key::Num0),
            Shortcut::ZoomFit => (Modifiers::CTRL | Modifiers::SHIFT, Key::Num0),
            Shortcut::ToggleGrid => (Modifiers::CTRL, Key::G),
            Shortcut::ToggleOnionSkin => (Modifiers::ALT, Key::O),
            Shortcut::ToggleFullscreen => (Modifiers::NONE, Key::F11),

            // Sprite
            Shortcut::SpriteProperties => (Modifiers::CTRL | Modifiers::ALT, Key::S),
            Shortcut::CanvasSize => (Modifiers::CTRL | Modifiers::ALT, Key::C),
            Shortcut::ResizeImage => (Modifiers::CTRL | Modifiers::ALT, Key::I),

            // Layer
            Shortcut::NewLayer => (Modifiers::CTRL | Modifiers::SHIFT, Key::N),
            Shortcut::DeleteLayer => (Modifiers::CTRL | Modifiers::SHIFT, Key::Delete),
            Shortcut::DuplicateLayer => (Modifiers::CTRL, Key::J),
            Shortcut::MergeDown => (Modifiers::CTRL, Key::E),
            Shortcut::GroupLayers => (Modifiers::CTRL, Key::G),

            // Frame
            Shortcut::NewFrame => (Modifiers::ALT, Key::N),
            Shortcut::DeleteFrame => (Modifiers::ALT, Key::Delete),
            Shortcut::DuplicateFrame => (Modifiers::ALT, Key::D),
            Shortcut::PrevFrame => (Modifiers::NONE, Key::ArrowLeft),
            Shortcut::NextFrame => (Modifiers::NONE, Key::ArrowRight),
            Shortcut::FirstFrame => (Modifiers::CTRL, Key::Home),
            Shortcut::LastFrame => (Modifiers::CTRL, Key::End),

            // Tools
            Shortcut::ToolPencil => (Modifiers::NONE, Key::B),
            Shortcut::ToolEraser => (Modifiers::NONE, Key::E),
            Shortcut::ToolEyedropper => (Modifiers::NONE, Key::I),
            Shortcut::ToolMove => (Modifiers::NONE, Key::V),
            Shortcut::ToolZoom => (Modifiers::NONE, Key::Z),
            Shortcut::ToolHand => (Modifiers::NONE, Key::H),
            Shortcut::ToolSelection => (Modifiers::NONE, Key::M),
            Shortcut::ToolPaintBucket => (Modifiers::NONE, Key::G),
            Shortcut::ToolLine => (Modifiers::NONE, Key::L),
            Shortcut::ToolRectangle => (Modifiers::NONE, Key::U),
            Shortcut::ToolEllipse => (Modifiers::NONE, Key::O),
            Shortcut::ToolSpray => (Modifiers::NONE, Key::S),
            Shortcut::ToolText => (Modifiers::NONE, Key::T),

            // Playback
            Shortcut::PlayPause => (Modifiers::NONE, Key::Space),
            Shortcut::Stop => (Modifiers::CTRL, Key::Space),
            Shortcut::Loop => (Modifiers::CTRL, Key::L),
            Shortcut::OnionSkinPrev => (Modifiers::ALT, Key::ArrowLeft),
            Shortcut::OnionSkinNext => (Modifiers::ALT, Key::ArrowRight),

            // Palette
            Shortcut::ForegroundColor => (Modifiers::NONE, Key::X),
            Shortcut::BackgroundColor => (Modifiers::NONE, Key::C),
            Shortcut::SwapColors => (Modifiers::NONE, Key::X), // Same as foreground
            Shortcut::DefaultColors => (Modifiers::NONE, Key::D),
        })
    }

    pub fn description(&self) -> &'static str {
        match self {
            Shortcut::New => "New sprite",
            Shortcut::Open => "Open file",
            Shortcut::Save => "Save",
            Shortcut::SaveAs => "Save as",
            Shortcut::Export => "Export",
            Shortcut::Exit => "Exit",
            Shortcut::Undo => "Undo",
            Shortcut::Redo => "Redo",
            Shortcut::Cut => "Cut",
            Shortcut::Copy => "Copy",
            Shortcut::Paste => "Paste",
            Shortcut::Delete => "Delete",
            Shortcut::SelectAll => "Select all",
            Shortcut::Deselect => "Deselect",
            Shortcut::ZoomIn => "Zoom in",
            Shortcut::ZoomOut => "Zoom out",
            Shortcut::ZoomReset => "Reset zoom",
            Shortcut::ZoomFit => "Fit to window",
            Shortcut::ToggleGrid => "Toggle grid",
            Shortcut::ToggleOnionSkin => "Toggle onion skin",
            Shortcut::ToggleFullscreen => "Toggle fullscreen",
            Shortcut::SpriteProperties => "Sprite properties",
            Shortcut::CanvasSize => "Canvas size",
            Shortcut::ResizeImage => "Resize image",
            Shortcut::NewLayer => "New layer",
            Shortcut::DeleteLayer => "Delete layer",
            Shortcut::DuplicateLayer => "Duplicate layer",
            Shortcut::MergeDown => "Merge down",
            Shortcut::GroupLayers => "Group layers",
            Shortcut::NewFrame => "New frame",
            Shortcut::DeleteFrame => "Delete frame",
            Shortcut::DuplicateFrame => "Duplicate frame",
            Shortcut::PrevFrame => "Previous frame",
            Shortcut::NextFrame => "Next frame",
            Shortcut::FirstFrame => "First frame",
            Shortcut::LastFrame => "Last frame",
            Shortcut::ToolPencil => "Pencil tool",
            Shortcut::ToolEraser => "Eraser tool",
            Shortcut::ToolEyedropper => "Eyedropper tool",
            Shortcut::ToolMove => "Move tool",
            Shortcut::ToolZoom => "Zoom tool",
            Shortcut::ToolHand => "Hand tool",
            Shortcut::ToolSelection => "Selection tool",
            Shortcut::ToolPaintBucket => "Paint bucket tool",
            Shortcut::ToolLine => "Line tool",
            Shortcut::ToolRectangle => "Rectangle tool",
            Shortcut::ToolEllipse => "Ellipse tool",
            Shortcut::ToolSpray => "Spray tool",
            Shortcut::ToolText => "Text tool",
            Shortcut::PlayPause => "Play/Pause",
            Shortcut::Stop => "Stop",
            Shortcut::Loop => "Toggle loop",
            Shortcut::OnionSkinPrev => "Previous onion skin frame",
            Shortcut::OnionSkinNext => "Next onion skin frame",
            Shortcut::ForegroundColor => "Foreground color",
            Shortcut::BackgroundColor => "Background color",
            Shortcut::SwapColors => "Swap colors",
            Shortcut::DefaultColors => "Default colors",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Shortcut::New | Shortcut::Open | Shortcut::Save | Shortcut::SaveAs | Shortcut::Export | Shortcut::Exit => "File",
            Shortcut::Undo | Shortcut::Redo | Shortcut::Cut | Shortcut::Copy | Shortcut::Paste | Shortcut::Delete | Shortcut::SelectAll | Shortcut::Deselect => "Edit",
            Shortcut::ZoomIn | Shortcut::ZoomOut | Shortcut::ZoomReset | Shortcut::ZoomFit | Shortcut::ToggleGrid | Shortcut::ToggleOnionSkin | Shortcut::ToggleFullscreen => "View",
            Shortcut::SpriteProperties | Shortcut::CanvasSize | Shortcut::ResizeImage => "Sprite",
            Shortcut::NewLayer | Shortcut::DeleteLayer | Shortcut::DuplicateLayer | Shortcut::MergeDown | Shortcut::GroupLayers => "Layer",
            Shortcut::NewFrame | Shortcut::DeleteFrame | Shortcut::DuplicateFrame | Shortcut::PrevFrame | Shortcut::NextFrame | Shortcut::FirstFrame | Shortcut::LastFrame => "Frame",
            Shortcut::ToolPencil | Shortcut::ToolEraser | Shortcut::ToolEyedropper | Shortcut::ToolMove | Shortcut::ToolZoom | Shortcut::ToolHand | Shortcut::ToolSelection | Shortcut::ToolPaintBucket | Shortcut::ToolLine | Shortcut::ToolRectangle | Shortcut::ToolEllipse | Shortcut::ToolSpray | Shortcut::ToolText => "Tools",
            Shortcut::PlayPause | Shortcut::Stop | Shortcut::Loop | Shortcut::OnionSkinPrev | Shortcut::OnionSkinNext => "Playback",
            Shortcut::ForegroundColor | Shortcut::BackgroundColor | Shortcut::SwapColors | Shortcut::DefaultColors => "Palette",
        }
    }

    pub fn all() -> &'static [Shortcut] {
        &[
            Shortcut::New, Shortcut::Open, Shortcut::Save, Shortcut::SaveAs, Shortcut::Export, Shortcut::Exit,
            Shortcut::Undo, Shortcut::Redo, Shortcut::Cut, Shortcut::Copy, Shortcut::Paste, Shortcut::Delete, Shortcut::SelectAll, Shortcut::Deselect,
            Shortcut::ZoomIn, Shortcut::ZoomOut, Shortcut::ZoomReset, Shortcut::ZoomFit, Shortcut::ToggleGrid, Shortcut::ToggleOnionSkin, Shortcut::ToggleFullscreen,
            Shortcut::SpriteProperties, Shortcut::CanvasSize, Shortcut::ResizeImage,
            Shortcut::NewLayer, Shortcut::DeleteLayer, Shortcut::DuplicateLayer, Shortcut::MergeDown, Shortcut::GroupLayers,
            Shortcut::NewFrame, Shortcut::DeleteFrame, Shortcut::DuplicateFrame, Shortcut::PrevFrame, Shortcut::NextFrame, Shortcut::FirstFrame, Shortcut::LastFrame,
            Shortcut::ToolPencil, Shortcut::ToolEraser, Shortcut::ToolEyedropper, Shortcut::ToolMove, Shortcut::ToolZoom, Shortcut::ToolHand, Shortcut::ToolSelection, Shortcut::ToolPaintBucket, Shortcut::ToolLine, Shortcut::ToolRectangle, Shortcut::ToolEllipse, Shortcut::ToolSpray, Shortcut::ToolText,
            Shortcut::PlayPause, Shortcut::Stop, Shortcut::Loop, Shortcut::OnionSkinPrev, Shortcut::OnionSkinNext,
            Shortcut::ForegroundColor, Shortcut::BackgroundColor, Shortcut::SwapColors, Shortcut::DefaultColors,
        ]
    }
}

/// Handle keyboard shortcuts for the app
impl OverSpriteApp {
    pub fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for shortcut in Shortcut::all() {
                if let Some((modifiers, key)) = shortcut.default_key() {
                    if i.modifiers.matches_logically(modifiers) && i.key_pressed(key) {
                        self.execute_shortcut(*shortcut);
                    }
                }
            }
        });
    }

    fn execute_shortcut(&mut self, shortcut: Shortcut) {
        match shortcut {
            Shortcut::New => self.new_sprite_dialog(),
            Shortcut::Open => self.open_file_dialog(),
            Shortcut::Save => self.save_file(),
            Shortcut::SaveAs => self.save_as_dialog(),
            Shortcut::Export => self.export_dialog(),
            Shortcut::Exit => std::process::exit(0),

            Shortcut::Undo => {}, // TODO: Implement undo
            Shortcut::Redo => {}, // TODO: Implement redo

            Shortcut::ZoomIn => self.zoom *= 1.2,
            Shortcut::ZoomOut => self.zoom /= 1.2,
            Shortcut::ZoomReset => self.zoom = 1.0,
            Shortcut::ToggleGrid => self.show_grid = !self.show_grid,
            Shortcut::ToggleOnionSkin => self.show_onion_skin = !self.show_onion_skin,

            Shortcut::NewLayer => self.add_layer(),
            Shortcut::DeleteLayer => self.delete_selected_layer(),

            Shortcut::PrevFrame => {
                if self.current_frame > 0 { self.current_frame -= 1; }
            }
            Shortcut::NextFrame => {
                if let Some(sprite) = &self.sprite {
                    if self.current_frame + 1 < sprite.frames.order.len() {
                        self.current_frame += 1;
                    }
                }
            }

            Shortcut::ToolPencil => self.current_tool = crate::Tool::Pencil,
            Shortcut::ToolEraser => self.current_tool = crate::Tool::Eraser,
            Shortcut::ToolEyedropper => self.current_tool = crate::Tool::Eyedropper,
            Shortcut::ToolMove => self.current_tool = crate::Tool::Move,
            Shortcut::ToolZoom => self.current_tool = crate::Tool::Zoom,
            Shortcut::ToolHand => self.current_tool = crate::Tool::Hand,
            Shortcut::ToolSelection => self.current_tool = crate::Tool::Selection,
            Shortcut::ToolPaintBucket => self.current_tool = crate::Tool::PaintBucket,
            Shortcut::ToolLine => self.current_tool = crate::Tool::Line,
            Shortcut::ToolRectangle => self.current_tool = crate::Tool::Rectangle,
            Shortcut::ToolEllipse => self.current_tool = crate::Tool::Ellipse,
            Shortcut::ToolSpray => self.current_tool = crate::Tool::Spray,
            Shortcut::ToolText => self.current_tool = crate::Tool::Text,

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_descriptions() {
        assert_eq!(Shortcut::New.description(), "New sprite");
        assert_eq!(Shortcut::Save.description(), "Save");
    }

    #[test]
    fn test_shortcut_categories() {
        assert_eq!(Shortcut::New.category(), "File");
        assert_eq!(Shortcut::Undo.category(), "Edit");
        assert_eq!(Shortcut::ToolPencil.category(), "Tools");
    }

    #[test]
    fn test_all_shortcuts_count() {
        assert_eq!(Shortcut::all().len(), 58);
    }
}