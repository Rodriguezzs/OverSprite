//! OverSprite Editor - Main entry point for the GUI application

use oversprite_ui::{OverSpriteApp, theme::{apply_theme, setup_fonts, Theme}};

fn main() -> eframe::Result<()> {
    // Configure the native options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("OverSprite - Modern Pixel Art Editor"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "OverSprite",
        native_options,
        Box::new(|cc| {
            // Setup custom fonts
            setup_fonts(&cc.egui_ctx);

            // Apply the Modern Dark theme
            apply_theme(&cc.egui_ctx, Theme::ModernDark);

            // Create the app
            Ok(Box::new(OverSpriteApp::new()))
        }),
    )
}