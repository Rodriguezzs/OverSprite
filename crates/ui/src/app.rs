//! OverSprite UI - App Module
//!
//! Main application entry point and layout management with modern dock-based UI.

use eframe::egui;
use oversprite_core::{CoreResult, Sprite, ColorMode, Size, LayerType, Uuid};
use crate::{OverSpriteApp, Tool, theme};

pub fn run_app() -> CoreResult<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 720.0])
            .with_title("OverSprite")
            .with_decorations(true)
            .with_transparent(false)
            .with_drag_and_drop(true),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "OverSprite",
        options,
        Box::new(|cc| {
            // Setup fonts and modern dark theme
            theme::setup_fonts(&cc.egui_ctx);
            theme::apply_theme(&cc.egui_ctx, theme::Theme::ModernDark);

            Ok(Box::new(OverSpriteApp::new()))
        }),
    ).map_err(|e| oversprite_core::CoreError::Internal(format!("Failed to run app: {}", e)))?;

    Ok(())
}

pub fn create_new_sprite(width: u32, height: u32, color_mode: ColorMode) -> Sprite {
    Sprite::new_with_size(width, height, color_mode)
}

impl OverSpriteApp {
    /// Main UI layout - Modern dock-based workspace
    pub fn layout_ui(&mut self, ctx: &egui::Context) {
        // ─── Top: Menu Bar ──────────────────────────────────────────
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(theme::spacing::MENU_BAR_HEIGHT)
            .show(ctx, |ui| {
                self.show_menu_bar(ui);
            });

        // ─── Floating Contextual Toolbar (appears below menu bar) ───
        if self.ui_state.toolbar_visible {
            self.show_contextual_toolbar(ctx);
        }

        // ─── Left Dock: Layers ──────────────────────────────────────
        let layers_width = if self.ui_state.layers_dock_collapsed {
            theme::spacing::PANEL_MIN_WIDTH
        } else {
            theme::spacing::PANEL_DEFAULT_WIDTH
        };

        egui::SidePanel::left("layers_dock")
            .resizable(true)
            .default_width(layers_width)
            .width_range(theme::spacing::PANEL_MIN_WIDTH..=theme::spacing::PANEL_MAX_WIDTH)
            .show_animated(ctx, self.ui_state.layers_dock_open, |ui| {
                self.show_layers_dock(ui);
            });

        // ─── Right Dock: Properties / Palette ───────────────────────
        let props_width = if self.ui_state.properties_dock_collapsed {
            theme::spacing::PANEL_MIN_WIDTH
        } else {
            theme::spacing::PANEL_DEFAULT_WIDTH
        };

        egui::SidePanel::right("properties_dock")
            .resizable(true)
            .default_width(props_width)
            .width_range(theme::spacing::PANEL_MIN_WIDTH..=theme::spacing::PANEL_MAX_WIDTH)
            .show_animated(ctx, self.ui_state.properties_dock_open, |ui| {
                self.show_properties_dock(ui);
            });

        // ─── Bottom Dock: Timeline ──────────────────────────────────
        let timeline_height = if self.ui_state.timeline_dock_collapsed {
            theme::spacing::TIMELINE_MIN_HEIGHT
        } else {
            theme::spacing::TIMELINE_DEFAULT_HEIGHT
        };

        egui::TopBottomPanel::bottom("timeline_dock")
            .resizable(true)
            .default_height(timeline_height)
            .height_range(theme::spacing::TIMELINE_MIN_HEIGHT..=theme::spacing::TIMELINE_MAX_HEIGHT)
            .show_animated(ctx, self.ui_state.timeline_dock_open, |ui| {
                self.show_timeline_dock(ui);
            });

        // ─── Center: Viewport (The Protagonist) ─────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::colors::BG_BASE))
            .show(ctx, |ui| {
                self.show_viewport(ui);
            });
    }

    /// Floating contextual toolbar - appears centered below menu bar
    /// Shows tool-specific options (brush size, opacity, blend mode, etc.)
    fn show_contextual_toolbar(&mut self, ctx: &egui::Context) {
        use theme::spacing::TOOLBAR_HEIGHT;

        egui::Area::new("contextual_toolbar".into())
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, theme::spacing::MENU_BAR_HEIGHT + 4.0))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0f32, theme::colors::BORDER))
                    .rounding(theme::spacing::RADIUS_LG)
                    .inner_margin(egui::Margin::symmetric(theme::spacing::LG, theme::spacing::SM))
                    .shadow(egui::epaint::Shadow {
                        color: theme::colors::SHADOW_KEY,
                        offset: egui::vec2(0.0, 8.0),
                        blur: 24.0f32,
                        spread: 4.0f32,
                    })
                    .show(ui, |ui| {
                        ui.set_min_height(TOOLBAR_HEIGHT);
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing.x = theme::spacing::MD;
                            self.show_tool_options(ui);
                        });
                    });
            });
    }

    /// Tool-specific options in the contextual toolbar
    fn show_tool_options(&mut self, ui: &mut egui::Ui) {
        match self.current_tool {
            Tool::Pencil | Tool::Eraser => {
                // Brush size
                ui.add(egui::DragValue::new(&mut 8u32)
                    .range(1..=512)
                    .speed(1.0)
                    .prefix("Size: "));

                ui.separator();

                // Opacity
                ui.add(egui::DragValue::new(&mut 255u8)
                    .range(0..=255)
                    .speed(1.0)
                    .prefix("Opacity: "));

                ui.separator();

                // Hardness (for pencil)
                if self.current_tool == Tool::Pencil {
                    ui.add(egui::Slider::new(&mut 1.0f32, 0.0..=1.0)
                        .text("Hardness")
                        .show_value(true));
                }

                ui.separator();

                // Blend mode (compact)
                let current_tool = self.current_tool;
                egui::ComboBox::new("toolbar_blend_mode", "Normal")
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for mode in oversprite_core::BlendMode::all_modes().iter().take(8) {
                            ui.selectable_value(&mut self.current_tool, current_tool, format!("{:?}", mode));
                        }
                    });
            }
            Tool::Selection => {
                // Transform options
                if ui.button("🔄 Transform").clicked() {}
                ui.separator();
                if ui.button("⤢ Flip H").clicked() {}
                if ui.button("⤡ Flip V").clicked() {}
                ui.separator();
                if ui.button("⟲ Rotate 90°").clicked() {}
                if ui.button("⟳ Rotate -90°").clicked() {}
                ui.separator();
                if ui.button("⬜ Crop to Selection").clicked() {}
            }
            Tool::Move => {
                ui.label("Click-drag to move layer");
                ui.separator();
                if ui.button("Align Center").clicked() {}
                if ui.button("Distribute").clicked() {}
            }
            Tool::Zoom => {
                ui.label("Scroll to zoom, Click to zoom in");
                ui.separator();
                if ui.button("100%").clicked() { self.zoom = 1.0; }
                if ui.button("Fit").clicked() { self.zoom = 0.5; }
            }
            Tool::Hand => {
                ui.label("Space + Drag to pan");
            }
            Tool::Eyedropper => {
                ui.label("Click to sample color");
                ui.separator();
                ui.checkbox(&mut false, "Sample all layers");
            }
            Tool::PaintBucket => {
                ui.add(egui::DragValue::new(&mut 32u8)
                    .range(0..=255)
                    .prefix("Tolerance: "));
                ui.separator();
                ui.checkbox(&mut false, "Contiguous");
                ui.checkbox(&mut false, "All layers");
            }
            Tool::Line | Tool::Rectangle | Tool::Ellipse => {
                ui.add(egui::DragValue::new(&mut 2u32)
                    .range(1..=64)
                    .prefix("Stroke: "));
                ui.separator();
                ui.checkbox(&mut false, "Fill");
                ui.checkbox(&mut false, "From center");
            }
            Tool::Spray => {
                ui.add(egui::DragValue::new(&mut 16u32)
                    .range(1..=128)
                    .prefix("Radius: "));
                ui.separator();
                ui.add(egui::DragValue::new(&mut 10u32)
                    .range(1..=100)
                    .prefix("Density: "));
            }
            Tool::Text => {
                ui.label("Click canvas to add text");
                ui.separator();
                ui.add(egui::DragValue::new(&mut 16.0f32)
                    .range(6.0..=128.0)
                    .prefix("Size: "));
            }
        }
    }

    /// Layers Dock - Collapsible, modern layer management
    fn show_layers_dock(&mut self, ui: &mut egui::Ui) {
        let collapsed = self.ui_state.layers_dock_collapsed;

        // Header with collapse toggle
        ui.horizontal(|ui| {
            if !collapsed {
                ui.heading("Layers");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Collapse/expand button
                let icon = if collapsed { "▶" } else { "◀" };
                if ui.small_button(icon).on_hover_text(if collapsed { "Expand" } else { "Collapse" }).clicked() {
                    self.ui_state.layers_dock_collapsed = !collapsed;
                }

                if !collapsed {
                    if ui.button("+").on_hover_text("New Layer (Ctrl+Shift+N)").clicked() {
                        self.add_layer();
                    }
                }
            });
        });

        ui.add_space(theme::spacing::SM);

        if collapsed {
            // Icon-only mode: show layer icons as compact stack
            if let Some(sprite) = &self.sprite {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            for layer_uuid in &sprite.layers.order {
                                if let Some(layer) = sprite.layer_data.get(layer_uuid) {
                                    let selected = self.ui_state.layers_panel_selected == Some(*layer_uuid);
                                    let response = ui.add(
                                        egui::Label::new(layer_icon(layer))
                                            .selectable(selected)
                                            .sense(egui::Sense::click())
                                    ).on_hover_text(&layer.name);

                                    if response.clicked() {
                                        self.ui_state.layers_panel_selected = Some(*layer_uuid);
                                    }
                                }
                            }
                        });
                    });
            }
            return;
        }

        // Expanded mode: Full layer list with controls
        let layer_uuids: Vec<Uuid> = if let Some(sprite) = &self.sprite {
            sprite.layers.order.iter().cloned().collect()
        } else {
            Vec::new()
        };

        if !layer_uuids.is_empty() {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for layer_uuid in &layer_uuids {
                        self.show_layer_item_modern(ui, *layer_uuid);
                    }
                });
        }

        ui.add_space(theme::spacing::SM);
        ui.separator();
        ui.add_space(theme::spacing::XS);

        // New layer button (prominent in expanded mode)
        if ui.button("➕  New Layer").clicked() {
            self.add_layer();
        }
    }

    /// Modern layer item with inline controls (by UUID to avoid borrow conflicts)
    fn show_layer_item_modern(&mut self, ui: &mut egui::Ui, layer_uuid: Uuid) {
        let (_selected, is_group, layer_name, layer_type, visible, opacity) = {
            if let Some(sprite) = &self.sprite {
                if let Some(layer) = sprite.layer_data.get(&layer_uuid) {
                    (self.ui_state.layers_panel_selected == Some(layer.uuid),
                     layer.layer_type == oversprite_core::LayerType::Group,
                     layer.name.clone(),
                     layer.layer_type,
                     layer.visible,
                     layer.opacity)
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let icon = Self::layer_icon_from_type(layer_type);

        ui.horizontal(|ui| {
            // Drag handle (for reordering - future)
            ui.label("⋮").on_hover_text("Drag to reorder");

            // Visibility toggle
            let mut visible = visible;
            if ui.checkbox(&mut visible, "").changed() {
                if let Some(sprite) = &mut self.sprite {
                    if let Some(l) = sprite.layer_data.get_mut(&layer_uuid) {
                        l.visible = visible;
                    }
                }
            }

            // Layer type icon
            ui.label(icon).on_hover_text(format!("{:?}", layer_type));

            // Layer name (editable on double-click)
            let mut name = layer_name;
            let name_response = ui.add(
                egui::TextEdit::singleline(&mut name)
                    .frame(false)
                    .desired_width(f32::INFINITY)
            );
            if name_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Commit rename
                if let Some(sprite) = &mut self.sprite {
                    if let Some(l) = sprite.layer_data.get_mut(&layer_uuid) {
                        l.name = name;
                    }
                }
            }

            if name_response.clicked() {
                self.ui_state.layers_panel_selected = Some(layer_uuid);
            }

            // Opacity slider (compact)
            if !collapsed_placeholder() {
                let mut opacity = opacity;
                let response = ui.add(
                    egui::Slider::new(&mut opacity, 0..=255)
                        .text("Opacity")
                        .show_value(true)
                        .trailing_fill(true)
                );
                if response.changed() {
                    if let Some(sprite) = &mut self.sprite {
                        if let Some(l) = sprite.layer_data.get_mut(&layer_uuid) {
                            l.opacity = opacity;
                        }
                    }
                }
            }
        });

        // Context menu
        ui.allocate_response(egui::vec2(0.0, 0.0), egui::Sense::hover())
            .context_menu(|ui| {
                if ui.button("Duplicate").clicked() {}
                if ui.button("Delete").clicked() { self.delete_selected_layer(); }
                ui.separator();
                if ui.button("Merge Down").clicked() {}
                if ui.button("Convert to Tilemap").clicked() {}
                if is_group {
                    if ui.button("Ungroup").clicked() {}
                } else {
                    if ui.button("Group Selected").clicked() {}
                }
            });
    }

    fn layer_icon_from_type(layer_type: oversprite_core::LayerType) -> &'static str {
        match layer_type {
            oversprite_core::LayerType::Image => "🖼",
            oversprite_core::LayerType::Group => "📁",
            oversprite_core::LayerType::Tilemap => "🗺",
        }
    }

    /// Properties Dock - Contextual, shows relevant properties for selection
    fn show_properties_dock(&mut self, ui: &mut egui::Ui) {
        let collapsed = self.ui_state.properties_dock_collapsed;

        ui.horizontal(|ui| {
            if !collapsed {
                ui.heading("Properties");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let icon = if collapsed { "▶" } else { "◀" };
                if ui.small_button(icon).on_hover_text(if collapsed { "Expand" } else { "Collapse" }).clicked() {
                    self.ui_state.properties_dock_collapsed = !collapsed;
                }
            });
        });

        ui.add_space(theme::spacing::SM);

        if collapsed {
            // Show minimal tool indicator
            ui.centered_and_justified(|ui| {
                ui.label(tool_icon(self.current_tool));
            });
            return;
        }

        // Contextual properties based on selection
        let (layer_uuid, frame_uuid, has_sprite) = {
            if let Some(_sprite) = &self.sprite {
                (self.ui_state.layers_panel_selected, self.ui_state.frames_panel_selected, true)
            } else {
                (None, None, false)
            }
        };

        if has_sprite {
            if let Some(uuid) = layer_uuid {
                self.show_layer_properties_modern_by_uuid(ui, uuid);
            } else if let Some(uuid) = frame_uuid {
                self.show_frame_properties_modern_by_uuid(ui, uuid);
            } else {
                self.show_document_properties_modern(ui);
            }
        }
    }

    fn show_document_properties_modern(&mut self, ui: &mut egui::Ui) {
        if let Some(sprite) = &self.sprite {
            egui::Grid::new("doc_props")
                .num_columns(2)
                .spacing([theme::spacing::MD, theme::spacing::SM])
                .show(ui, |ui| {
                    ui.label("Dimensions");
                    ui.label(format!("{} × {}", sprite.metadata.dimensions.w, sprite.metadata.dimensions.h));
                    ui.end_row();

                    ui.label("Color Mode");
                    ui.label(format!("{:?}", sprite.metadata.color_mode));
                    ui.end_row();

                    ui.label("Layers");
                    ui.label(format!("{}", sprite.layer_data.len()));
                    ui.end_row();

                    ui.label("Frames");
                    ui.label(format!("{}", sprite.frame_data.len()));
                    ui.end_row();

                ui.label("Default Frame Duration");
                ui.label(format!("{} ms", sprite.metadata.default_frame_duration_ms));
                ui.end_row();
            });
        }
    }

    /// Timeline Dock - Modern animation tracks (Figma/video editor style)
    fn show_timeline_dock(&mut self, ui: &mut egui::Ui) {
        let collapsed = self.ui_state.timeline_dock_collapsed;

        ui.horizontal(|ui| {
            if !collapsed {
                ui.heading("Timeline");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let icon = if collapsed { "▲" } else { "▼" };
                if ui.small_button(icon).on_hover_text(if collapsed { "Expand" } else { "Collapse" }).clicked() {
                    self.ui_state.timeline_dock_collapsed = !collapsed;
                }

                if !collapsed {
                    // Playback controls
                    ui.add_space(theme::spacing::SM);
                    self.show_playback_controls(ui);
                }
            });
        });

        if collapsed {
            // Minimal: just playback controls
            ui.add_space(theme::spacing::XS);
            self.show_playback_controls(ui);
            return;
        }

        ui.add_space(theme::spacing::SM);

        // Timeline tracks area
        if let Some(sprite) = &self.sprite {
            let clicked_frame = egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.show_timeline_tracks(ui, sprite)
                })
                .inner;

            if let Some(frame) = clicked_frame {
                self.current_frame = frame;
            }
        }
    }

    /// Minimal playback controls
    fn show_playback_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::spacing::XS;

            // Transport
            if ui.small_button("⏮").on_hover_text("First Frame").clicked() { self.current_frame = 0; }
            if ui.small_button("⏪").on_hover_text("Previous Frame").clicked() {
                if self.current_frame > 0 { self.current_frame -= 1; }
            }
            if self.playing {
                if ui.small_button("⏸").on_hover_text("Pause").clicked() { self.playing = false; }
            } else {
                if ui.small_button("▶").on_hover_text("Play").clicked() { self.playing = true; }
            }
            if ui.small_button("⏩").on_hover_text("Next Frame").clicked() {
                if let Some(sprite) = &self.sprite {
                    if self.current_frame + 1 < sprite.frames.order.len() {
                        self.current_frame += 1;
                    }
                }
            }
            if ui.small_button("⏭").on_hover_text("Last Frame").clicked() {
                if let Some(sprite) = &self.sprite {
                    self.current_frame = sprite.frames.order.len().saturating_sub(1);
                }
            }

            ui.add_space(theme::spacing::MD);

            // FPS
            ui.label("FPS:");
            ui.add(egui::DragValue::new(&mut 24u32).range(1..=60).speed(1.0));

            ui.add_space(theme::spacing::SM);

            // Loop & Onion skin
            ui.checkbox(&mut self.play_loop, "Loop");
            ui.checkbox(&mut self.ui_state.show_onion_skin, "Onion Skin");
            if self.ui_state.show_onion_skin {
                ui.add(egui::DragValue::new(&mut self.ui_state.onion_skin_frames)
                    .range(1..=5)
                    .prefix("±"));
            }
        });
    }

    /// Modern timeline tracks - horizontal tracks with draggable clips
    fn show_timeline_tracks(&self, ui: &mut egui::Ui, sprite: &Sprite) -> Option<usize> {
        let frame_count = sprite.frames.order.len();
        let _track_height = 40.0;
        let frame_width = 60.0; // Base width per frame (scales with duration)
        let mut clicked_frame: Option<usize> = None;

        // Header row with frame numbers
        ui.horizontal(|ui| {
            ui.add_space(120.0); // Track header width
            for i in 0..frame_count {
                let frame_uuid = &sprite.frames.order[i];
                let frame = sprite.frame_data.get(frame_uuid);
                let duration = frame.map(|f| f.duration_ms).unwrap_or(100);
                let width = (duration as f32 / 100.0 * frame_width).max(40.0);

                let response = ui.allocate_response(
                    egui::vec2(width, 24.0),
                    egui::Sense::click()
                );
                ui.painter().text(
                    response.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", i + 1),
                    egui::FontId::proportional(10.0),
                    theme::colors::TEXT_MUTED
                );
            }
        });

        ui.add_space(theme::spacing::XS);

        // Track rows - one per layer
        for layer_uuid in &sprite.layers.order {
            if let Some(layer) = sprite.layer_data.get(layer_uuid) {
                if let Some(frame) = self.show_timeline_track(ui, sprite, layer, frame_width) {
                    clicked_frame = Some(frame);
                }
            }
        }
        clicked_frame
    }

    fn show_timeline_track(&self, ui: &mut egui::Ui, sprite: &Sprite, layer: &oversprite_core::Layer, frame_width: f32) -> Option<usize> {
        let _frame_count = sprite.frames.order.len();
        let mut clicked_frame: Option<usize> = None;

        ui.horizontal(|ui| {
            // Track header
            let header_response = ui.allocate_response(
                egui::vec2(120.0, 40.0),
                egui::Sense::click()
            );

            // Track background
            ui.painter().rect_filled(
                header_response.rect.expand2(egui::vec2(0.0, -2.0)),
                theme::spacing::RADIUS_SM,
                if layer.visible { theme::colors::BG_ELEVATED } else { theme::colors::BG_SUNKEN }
            );

            // Track header content
            ui.horizontal(|ui| {
                // Visibility toggle
                let mut visible = layer.visible;
                if ui.checkbox(&mut visible, "").changed() {}

                // Lock
                let mut locked = layer.lock_movement;
                if ui.checkbox(&mut locked, "").changed() {}

                // Layer icon + name
                ui.label(layer_icon(layer));
                ui.add(
                    egui::Label::new(&layer.name)
                        .truncate()
                        .wrap()
                );
            });

            // Frame cells for this layer
            for (i, frame_uuid) in sprite.frames.order.iter().enumerate() {
                let has_cel = sprite.frame_data.get(frame_uuid)
                    .map(|f| f.cels.iter().any(|cu| {
                        sprite.cel_data.get(cu).map(|c| c.layer_uuid == layer.uuid).unwrap_or(false)
                    }))
                    .unwrap_or(false);

                let selected = i == self.current_frame;
                let track_height = 40.0;

                let response = ui.allocate_response(
                    egui::vec2(frame_width, track_height),
                    egui::Sense::click() | egui::Sense::drag()
                );

                // Cell background
                if has_cel {
                    let color = if selected {
                        theme::colors::ACCENT
                    } else {
                        theme::colors::BG_HOVER
                    };
                    ui.painter().rect_filled(
                        response.rect.shrink(2.0),
                        theme::spacing::RADIUS_SM,
                        color
                    );

                    // Frame duration indicator
                    if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                        if frame.duration_ms != 100 {
                            ui.painter().text(
                                response.rect.right_top() + egui::vec2(-4.0, 4.0),
                                egui::Align2::RIGHT_TOP,
                                format!("{}ms", frame.duration_ms),
                                egui::FontId::proportional(8.0),
                                theme::colors::TEXT_MUTED
                            );
                        }
                    }
                } else {
                    // Empty cell - subtle grid
                    ui.painter().rect_stroke(
                        response.rect.shrink(2.0),
                        theme::spacing::RADIUS_SM,
                        egui::Stroke::new(1.0f32, theme::colors::DIVIDER)
                    );
                }

                // Current frame indicator
                if selected {
                    ui.painter().rect_stroke(
                        response.rect.shrink(1.0),
                        theme::spacing::RADIUS_SM,
                        egui::Stroke::new(2.0f32, theme::colors::ACCENT)
                    );
                }

                // Interaction
                if response.clicked() {
                    clicked_frame = Some(i);
                }
                if response.dragged() {
                    // Drag to reorder frames (future)
                }
            }
        });
        clicked_frame
    }

    /// Viewport - The absolute protagonist
    fn show_viewport(&mut self, ui: &mut egui::Ui) {
        if let Some(sprite) = &self.sprite {
            let available_size = ui.available_size();

            // Canvas size with zoom
            let canvas_size = Size::new(
                (sprite.metadata.dimensions.w as f32 * self.zoom) as u32,
                (sprite.metadata.dimensions.h as f32 * self.zoom) as u32,
            );

            // Allocate canvas area
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(
                    available_size.x.min(canvas_size.w as f32),
                    available_size.y.min(canvas_size.h as f32)
                ),
                egui::Sense::drag() | egui::Sense::click()
            );

            // Draw checkerboard background
            let painter = ui.painter_at(rect);
            self.draw_checkerboard(&painter, rect);

            // Rulers (toggleable)
            if self.ui_state.show_rulers {
                self.draw_rulers(&painter, rect, sprite);
            }

            // Grid (toggleable)
            if self.ui_state.show_grid {
                self.draw_grid(&painter, rect, sprite);
            }

            // Render current frame using renderer
            if let Some(renderer) = &mut self.renderer {
                let onion_skin = if self.ui_state.show_onion_skin {
                    Some(oversprite_renderer::OnionSkinConfig {
                        enabled: true,
                        show_previous: true,
                        show_next: true,
                        previous_opacity: 0.3,
                        next_opacity: 0.3,
                        previous_color: oversprite_core::Rgba::new(255, 100, 100, 255),
                        next_color: oversprite_core::Rgba::new(100, 100, 255, 255),
                        max_frames: self.ui_state.onion_skin_frames,
                    })
                } else {
                    None
                };

                if let Err(e) = renderer.render_sprite(
                    sprite,
                    self.current_frame,
                    &sprite.layer_data,
                    &sprite.cel_data,
                    &sprite.tilesets,
                    &sprite.global_palette,
                    onion_skin.as_ref(),
                ) {
                    eprintln!("Render error: {}", e);
                }
            } else {
                // Fallback placeholder
                if let Some(frame_uuid) = sprite.frames.order.get(self.current_frame) {
                    if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                        self.render_frame_placeholder(&painter, rect, sprite, frame);
                    }
                }
            }

            // Draw the rendered pixmap to the egui painter
            if let Some(renderer) = &self.renderer {
                self.draw_renderer_pixmap(&painter, rect, renderer);
            }

            // Onion skin overlay
            if self.ui_state.show_onion_skin {
                self.draw_onion_skin(&painter, rect, sprite);
            }

            // Handle canvas interaction
            self.handle_canvas_interaction(&response, ui);

            // Show zoom level indicator (top-left overlay)
            self.show_zoom_indicator(ui, rect);
        } else {
            // Welcome state
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(egui::RichText::new("OverSprite")
                    .size(48.0)
                    .color(theme::colors::TEXT_PRIMARY)
                    .strong());
                ui.add_space(theme::spacing::LG);
                ui.label(egui::RichText::new("Modern Pixel Art & Animation Editor")
                    .size(16.0)
                    .color(theme::colors::TEXT_SECONDARY));
                ui.add_space(theme::spacing::XL);
                if ui.button("➕  New Sprite").clicked() {
                    self.new_sprite_dialog();
                }
                ui.add_space(theme::spacing::SM);
                if ui.button("📁  Open File").clicked() {
                    self.open_file_dialog();
                }
            });
        }
    }

    fn show_zoom_indicator(&self, ui: &mut egui::Ui, _rect: egui::Rect) {
        let zoom_pct = (self.zoom * 100.0).round() as u32;
        egui::Area::new("zoom_indicator".into())
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 12.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::none()
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0f32, theme::colors::BORDER))
                    .rounding(theme::spacing::RADIUS_MD)
                    .inner_margin(egui::Margin::symmetric(theme::spacing::SM, theme::spacing::XS))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("{}%", zoom_pct))
                            .size(11.0)
                            .color(theme::colors::TEXT_SECONDARY)
                            .monospace());
                    });
            });
    }

    // ─── Menu Bar ──────────────────────────────────────────────────
    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::spacing::SM;

            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Sprite…").clicked() { self.new_sprite_dialog(); }
                if ui.button("Open…").clicked() { self.open_file_dialog(); }
                ui.separator();
                if ui.button("Save").clicked() { self.save_file(); }
                if ui.button("Save As…").clicked() { self.save_as_dialog(); }
                ui.separator();
                if ui.button("Export…").clicked() { self.export_dialog(); }
                ui.separator();
                if ui.button("Exit").clicked() { std::process::exit(0); }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {}
                if ui.button("Redo").clicked() {}
                ui.separator();
                if ui.button("Preferences…").clicked() {}
            });

            // Sprite menu
            ui.menu_button("Sprite", |ui| {
                if ui.button("Sprite Properties…").clicked() {}
                if ui.button("Canvas Size…").clicked() {}
            });

            // Layer menu
            ui.menu_button("Layer", |ui| {
                if ui.button("New Layer").clicked() { self.add_layer(); }
                if ui.button("Delete Layer").clicked() { self.delete_selected_layer(); }
                ui.separator();
                if ui.button("Duplicate Layer").clicked() {}
                if ui.button("Merge Down").clicked() {}
            });

            // View menu
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.ui_state.show_grid, "Show Grid");
                ui.checkbox(&mut self.ui_state.show_rulers, "Show Rulers");
                ui.separator();
                ui.checkbox(&mut self.ui_state.show_onion_skin, "Onion Skin");
                ui.separator();
                if ui.button("Zoom In").clicked() { self.zoom *= 1.2; }
                if ui.button("Zoom Out").clicked() { self.zoom /= 1.2; }
                if ui.button("Reset Zoom").clicked() { self.zoom = 1.0; }
                ui.separator();
                // Panel toggles
                ui.checkbox(&mut self.ui_state.layers_dock_open, "Layers Dock");
                ui.checkbox(&mut self.ui_state.properties_dock_open, "Properties Dock");
                ui.checkbox(&mut self.ui_state.timeline_dock_open, "Timeline Dock");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Help menu
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {}
                    if ui.button("Shortcuts").clicked() {}
                });
            });
        });
    }

    // ─── Canvas Helpers ────────────────────────────────────────────
    fn draw_checkerboard(&self, painter: &egui::Painter, rect: egui::Rect) {
        let tile_size = 16.0 / self.zoom; // Scale with zoom
        let light = egui::Color32::from_gray(180);
        let dark = egui::Color32::from_gray(140);

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

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, _sprite: &Sprite) {
        let grid_size = 16.0 * self.zoom; // 16px grid scaled
        if grid_size < 8.0 { return; } // Don't draw if too small

        let stroke = egui::Stroke::new(0.5_f32, theme::colors::DIVIDER);

        // Vertical lines
        let start_x = (rect.left() / grid_size).floor() as i32;
        let end_x = (rect.right() / grid_size).ceil() as i32;
        for x in start_x..=end_x {
            let px = x as f32 * grid_size;
            painter.line_segment([
                egui::pos2(px, rect.top()),
                egui::pos2(px, rect.bottom())
            ], stroke);
        }

        // Horizontal lines
        let start_y = (rect.top() / grid_size).floor() as i32;
        let end_y = (rect.bottom() / grid_size).ceil() as i32;
        for y in start_y..=end_y {
            let py = y as f32 * grid_size;
            painter.line_segment([
                egui::pos2(rect.left(), py),
                egui::pos2(rect.right(), py)
            ], stroke);
        }
    }

    fn draw_rulers(&self, painter: &egui::Painter, rect: egui::Rect, _sprite: &Sprite) {
        let ruler_size = 20.0;
        let text_color = theme::colors::TEXT_MUTED;

        // Top ruler
        let top_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.right(), rect.top() + ruler_size)
        );
        painter.rect_filled(top_rect, 0.0, theme::colors::BG_ELEVATED);

        let step = 50.0 * self.zoom;
        let start = (rect.left() / step).floor() as i32;
        let end = (rect.right() / step).ceil() as i32;

        for i in start..=end {
            let x = i as f32 * step;
            let major = i % 5 == 0;
            let tick_h = if major { 16.0 } else { 8.0 };
            painter.line_segment([
                egui::pos2(x, rect.top() + ruler_size - tick_h),
                egui::pos2(x, rect.top() + ruler_size)
            ], egui::Stroke::new(1.0_f32, theme::colors::TEXT_MUTED));

            if major {
                let font = egui::FontId::proportional(9.0);
                painter.text(
                    egui::pos2(x + 2.0, rect.top() + 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}", (x / self.zoom) as i32),
                    font, text_color
                );
            }
        }

        // Left ruler
        let left_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.left() + ruler_size, rect.bottom())
        );
        painter.rect_filled(left_rect, 0.0, theme::colors::BG_ELEVATED);

        let start = (rect.top() / step).floor() as i32;
        let end = (rect.bottom() / step).ceil() as i32;

        for i in start..=end {
            let y = i as f32 * step;
            let major = i % 5 == 0;
            let tick_w = if major { 16.0 } else { 8.0 };
            painter.line_segment([
                egui::pos2(rect.left() + ruler_size - tick_w, y),
                egui::pos2(rect.left() + ruler_size, y)
            ], egui::Stroke::new(1.0_f32, theme::colors::TEXT_MUTED));

            if major {
                let font = egui::FontId::proportional(9.0);
                painter.text(
                    egui::pos2(rect.left() + 2.0, y + 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}", (y / self.zoom) as i32),
                    font, text_color
                );
            }
        }
    }

    fn draw_onion_skin(&self, painter: &egui::Painter, rect: egui::Rect, sprite: &Sprite) {
        let frames = self.ui_state.onion_skin_frames;
        let current = self.current_frame;

        // Previous frames (red tint)
        for i in 1..=frames {
            let idx = current.saturating_sub(i);
            if idx < sprite.frames.order.len() {
                if let Some(frame_uuid) = sprite.frames.order.get(idx) {
                    if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                        self.render_frame_placeholder(painter, rect, sprite, frame);
                    }
                }
            }
        }

        // Next frames (green tint) - would need different tint
        for i in 1..=frames {
            let idx = current + i;
            if idx < sprite.frames.order.len() {
                if let Some(frame_uuid) = sprite.frames.order.get(idx) {
                    if let Some(frame) = sprite.frame_data.get(frame_uuid) {
                        self.render_frame_placeholder(painter, rect, sprite, frame);
                    }
                }
            }
        }
    }

    fn render_frame_placeholder(&self, painter: &egui::Painter, rect: egui::Rect, _sprite: &Sprite, _frame: &oversprite_core::Frame) {
        // Placeholder - will be replaced by actual renderer integration
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0_f32, theme::colors::ACCENT));
    }

    fn draw_renderer_pixmap(&self, painter: &egui::Painter, rect: egui::Rect, renderer: &oversprite_renderer::Renderer) {
        let pixmap = renderer.pixmap();
        let width = pixmap.width();
        let height = pixmap.height();

        // Convert tiny-skia pixmap to egui ColorImage
        let pixels = pixmap.data();
        let mut color_image = egui::ColorImage::new([width as usize, height as usize], egui::Color32::TRANSPARENT);

        // Copy pixels (tiny-skia uses RGBA premultiplied, egui expects unpremultiplied)
        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
            if i >= color_image.pixels.len() { break; }
            // tiny-skia stores premultiplied RGBA, need to unpremultiply for egui
            let a = pixel[3] as f32 / 255.0;
            let r = if a > 0.0 { (pixel[0] as f32 / a).round() as u8 } else { 0 };
            let g = if a > 0.0 { (pixel[1] as f32 / a).round() as u8 } else { 0 };
            let b = if a > 0.0 { (pixel[2] as f32 / a).round() as u8 } else { 0 };
            color_image.pixels[i] = egui::Color32::from_rgba_unmultiplied(r, g, b, pixel[3]);
        }

        // Create texture and draw
        let texture = painter.ctx().load_texture("renderer_output", color_image, Default::default());
        painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    }

    fn handle_canvas_interaction(&mut self, response: &egui::Response, ui: &egui::Ui) {
        // Pan with Hand tool or Space+drag
        if response.dragged() && (self.current_tool == Tool::Hand || ui.input(|i| i.modifiers.alt)) {
            self.pan.x -= response.drag_delta().x as i32;
            self.pan.y -= response.drag_delta().y as i32;
        }

        // Pencil drawing
        if response.dragged() && self.current_tool == Tool::Pencil {
            // TODO: Implement drawing via renderer
        }

        // Zoom with scroll (when hovered)
        if response.hovered() {
            let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 && ui.input(|i| i.modifiers.ctrl) {
                self.zoom *= (1.0 + scroll * 0.01).clamp(0.05, 64.0);
            }
        }

        // Focus for keyboard shortcuts
        if response.clicked() {
            response.request_focus();
        }
    }

    // ─── Helper methods ────────────────────────────────────────────
    pub(crate) fn new_sprite_dialog(&self) {}
    pub(crate) fn open_file_dialog(&self) {}
    pub(crate) fn save_file(&self) {}
    pub(crate) fn save_as_dialog(&self) {}
    pub(crate) fn export_dialog(&self) {}

    pub(crate) fn add_layer(&mut self) {
        if let Some(sprite) = &mut self.sprite {
            let layer = oversprite_core::Layer::new_image(format!("Layer {}", sprite.layers.order.len() + 1));
            sprite.add_layer(layer, None);
        }
    }

    pub(crate) fn delete_selected_layer(&mut self) {
        if let (Some(sprite), Some(uuid)) = (&mut self.sprite, self.ui_state.layers_panel_selected) {
            let _ = sprite.remove_layer(&uuid);
        }
    }

    fn show_layer_properties_modern_by_uuid(&mut self, ui: &mut egui::Ui, layer_uuid: Uuid) {
        if let Some(sprite) = &mut self.sprite {
            if let Some(layer) = sprite.layer_data.get_mut(&layer_uuid) {
                egui::Grid::new("layer_props")
                    .num_columns(2)
                    .spacing([theme::spacing::MD, theme::spacing::SM])
                    .striped(true)
                    .show(ui, |ui| {
                        // Name
                        ui.label("Name");
                        ui.text_edit_singleline(&mut layer.name);
                        ui.end_row();

                        // Type badge
                        ui.label("Type");
                        ui.add(egui::Label::new(format!("{:?}", layer.layer_type))
                            .selectable(false));
                        ui.end_row();

                        // Blend mode (dropdown)
                        ui.label("Blend Mode");
                        egui::ComboBox::new("layer_blend_mode", format!("{:?}", layer.blend_mode))
                            .show_ui(ui, |ui| {
                                for mode in oversprite_core::BlendMode::all_modes() {
                                    ui.selectable_value(&mut layer.blend_mode, *mode, format!("{:?}", mode));
                                }
                            });
                        ui.end_row();

                        // Opacity slider
                        ui.label("Opacity");
                        ui.add(
                            egui::Slider::new(&mut layer.opacity, 0..=255)
                                .show_value(true)
                                .trailing_fill(true)
                        );
                        ui.end_row();

                        // Visibility
                        ui.label("Visible");
                        let mut visible = layer.visible;
                        if ui.checkbox(&mut visible, "").changed() {
                            layer.visible = visible;
                        }
                        ui.end_row();

                        // Lock states
                        ui.label("Lock Movement");
                        let mut locked = layer.lock_movement;
                        if ui.checkbox(&mut locked, "").changed() {
                            layer.lock_movement = locked;
                        }
                        ui.end_row();

                        ui.label("Lock Alpha");
                        let mut alpha_locked = false; // Not in core yet
                        if ui.checkbox(&mut alpha_locked, "").changed() {}
                        ui.end_row();
                    });
            }
        }
    }

    fn show_frame_properties_modern_by_uuid(&mut self, ui: &mut egui::Ui, frame_uuid: Uuid) {
        if let Some(sprite) = &mut self.sprite {
            if let Some(frame) = sprite.frame_data.get_mut(&frame_uuid) {
                egui::Grid::new("frame_props")
                    .num_columns(2)
                    .spacing([theme::spacing::MD, theme::spacing::SM])
                    .show(ui, |ui| {
                        ui.label("Duration");
                        ui.add(egui::DragValue::new(&mut frame.duration_ms)
                            .range(1..=10000)
                            .suffix(" ms")
                            .speed(1.0));
                        ui.end_row();

                        ui.label("Cels");
                        ui.label(format!("{}", frame.cels.len()));
                        ui.end_row();
                    });
            }
        }
    }
}

// ─── Helper functions ──────────────────────────────────────────────
fn layer_icon(layer: &oversprite_core::Layer) -> &'static str {
    match layer.layer_type {
        LayerType::Image => "🖼",
        LayerType::Group => "📁",
        LayerType::Tilemap => "🗺",
    }
}

fn tool_icon(tool: Tool) -> &'static str {
    match tool {
        Tool::Pencil => "✏",
        Tool::Eraser => "🧽",
        Tool::Eyedropper => "💧",
        Tool::Move => "↔",
        Tool::Zoom => "🔍",
        Tool::Hand => "✋",
        Tool::Selection => "▢",
        Tool::PaintBucket => "🪣",
        Tool::Line => "📏",
        Tool::Rectangle => "⬜",
        Tool::Ellipse => "⭘",
        Tool::Spray => "💨",
        Tool::Text => "📝",
    }
}

// Placeholder for collapsed state check in layer item
fn collapsed_placeholder() -> bool { false }