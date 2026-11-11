//! OverSprite UI - Theme Module
//!
//! Modern Dark theme with contemporary aesthetic - clean, minimalist, and elegant.
//! Inspired by VS Code Dark+, Figma, and modern design systems.

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    ModernDark,
    Light,
    System,
}

/// Modern Dark color palette - Contemporary, easy on the eyes
pub mod colors {
    use egui::Color32;

    // Base backgrounds - deep, muted purples for reduced eye strain
    pub const BG_BASE: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x2E);      // #1E1E2E - Main background
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x28, 0x28, 0x3E);  // #28283E - Panels, cards
    pub const BG_HOVER: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x5A);     // #3A3A5A - Hover states
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x4A, 0x4A, 0x6A);    // #4A4A6A - Active/pressed
    pub const BG_SUNKEN: Color32 = Color32::from_rgb(0x18, 0x18, 0x28);    // #181828 - Inset areas

    // Text colors - High contrast for readability
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xCD, 0xD6, 0xF4); // #CDD6F4 - Primary text
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xA6, 0xAD, 0xC8); // #A6ADC8 - Secondary text
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x7F, 0x84, 0x9C);   // #7F849C - Placeholders, disabled
    pub const TEXT_INVERSE: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x2E); // On accent backgrounds

    // Accent - Lavender/blue family, modern and distinctive
    pub const ACCENT: Color32 = Color32::from_rgb(0x89, 0xB4, 0xFA);       // #89B4FA - Primary accent
    pub const ACCENT_MUTED: Color32 = Color32::from_rgb(0x6A, 0x8C, 0xE6); // #6A8CE6 - Muted accent
    pub const ACCENT_SUBTLE: Color32 = Color32::from_rgba_premultiplied(0x89, 0xB4, 0xFA, 0x33); // 20% opacity
    pub const ACCENT_STRONG: Color32 = Color32::from_rgb(0x74, 0xA0, 0xE8); // #74A0E8 - Hover/active

    // Semantic colors
    pub const SUCCESS: Color32 = Color32::from_rgb(0xA6, 0xE3, 0xA1);      // #A6E3A1 - Green
    pub const SUCCESS_SUBTLE: Color32 = Color32::from_rgba_premultiplied(0xA6, 0xE3, 0xA1, 0x33);
    pub const WARNING: Color32 = Color32::from_rgb(0xF9, 0xE2, 0xAF);      // #F9E2AF - Yellow
    pub const WARNING_SUBTLE: Color32 = Color32::from_rgba_premultiplied(0xF9, 0xE2, 0xAF, 0x33);
    pub const ERROR: Color32 = Color32::from_rgb(0xF3, 0x8B, 0xA8);        // #F38BA8 - Red
    pub const ERROR_SUBTLE: Color32 = Color32::from_rgba_premultiplied(0xF3, 0x8B, 0xA8, 0x33);
    pub const INFO: Color32 = Color32::from_rgb(0x89, 0xD0, 0xE3);         // #89D0E3 - Cyan

    // Borders & dividers
    pub const BORDER: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x5A);       // #3A3A5A - Subtle borders
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x4A, 0x4A, 0x6A); // #4A4A6A - Focused borders
    pub const DIVIDER: Color32 = Color32::from_rgba_premultiplied(0x7F, 0x84, 0x9C, 0x40); // 25% opacity

    // Selection
    pub const SELECTION_BG: Color32 = Color32::from_rgba_premultiplied(0x89, 0xB4, 0xFA, 0x40); // 25%
    pub const SELECTION_STROKE: Color32 = Color32::from_rgb(0x89, 0xB4, 0xFA);

    // Scrollbars
    pub const SCROLLBAR_BG: Color32 = Color32::TRANSPARENT;
    pub const SCROLLBAR_THUMB: Color32 = Color32::from_rgba_premultiplied(0x7F, 0x84, 0x9C, 0x80);
    pub const SCROLLBAR_THUMB_HOVER: Color32 = Color32::from_rgba_premultiplied(0xA6, 0xAD, 0xC8, 0xCC);

    // Shadows
    pub const SHADOW_AMBIENT: Color32 = Color32::from_rgba_premultiplied(0x00, 0x00, 0x00, 0x60);
    pub const SHADOW_KEY: Color32 = Color32::from_rgba_premultiplied(0x00, 0x00, 0x00, 0x40);
}

/// Spacing & sizing constants for consistent rhythm
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;

    pub const PANEL_MIN_WIDTH: f32 = 48.0;   // Collapsed width (icon-only)
    pub const PANEL_DEFAULT_WIDTH: f32 = 280.0;
    pub const PANEL_MAX_WIDTH: f32 = 400.0;

    pub const TIMELINE_MIN_HEIGHT: f32 = 60.0;
    pub const TIMELINE_DEFAULT_HEIGHT: f32 = 180.0;
    pub const TIMELINE_MAX_HEIGHT: f32 = 360.0;

    pub const TOOLBAR_HEIGHT: f32 = 44.0;
    pub const MENU_BAR_HEIGHT: f32 = 28.0;

    pub const RADIUS_SM: f32 = 4.0;
    pub const RADIUS_MD: f32 = 8.0;
    pub const RADIUS_LG: f32 = 12.0;
    pub const RADIUS_FULL: f32 = 999.0;

    pub const ITEM_SPACING_X: f32 = 12.0;
    pub const ITEM_SPACING_Y: f32 = 8.0;
    pub const BUTTON_PADDING_X: f32 = 14.0;
    pub const BUTTON_PADDING_Y: f32 = 8.0;
}

/// Apply the Modern Dark theme to the egui context
pub fn apply_theme(ctx: &egui::Context, _theme: Theme) {
    let style = modern_dark_theme();
    ctx.set_style(style);
}

fn modern_dark_theme() -> egui::Style {
    let mut style = egui::Style::default();

    // ─── Visuals ─────────────────────────────────────────────────────
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(colors::TEXT_PRIMARY),

        // Widget styling using modern egui API (per-interaction)
        widgets: egui::style::Widgets {
            noninteractive: widget_visuals(
                colors::BG_ELEVATED,
                colors::TEXT_SECONDARY,
                colors::BORDER,
            ),
            inactive: widget_visuals(
                colors::BG_ELEVATED,
                colors::TEXT_PRIMARY,
                colors::BORDER,
            ),
            hovered: widget_visuals(
                colors::BG_HOVER,
                colors::TEXT_PRIMARY,
                colors::ACCENT,
            ),
            active: widget_visuals(
                colors::BG_ACTIVE,
                colors::TEXT_PRIMARY,
                colors::ACCENT_STRONG,
            ),
            open: widget_visuals(
                colors::BG_ELEVATED,
                colors::TEXT_PRIMARY,
                colors::BORDER,
            ),
        },

        selection: egui::style::Selection {
            bg_fill: colors::SELECTION_BG,
            stroke: egui::Stroke::new(1.0_f32, colors::SELECTION_STROKE),
        },

        hyperlink_color: colors::ACCENT,
        faint_bg_color: colors::BG_BASE,
        extreme_bg_color: colors::BG_SUNKEN,
        code_bg_color: colors::BG_ELEVATED,

        warn_fg_color: colors::WARNING,
        error_fg_color: colors::ERROR,

        // Window & panel styling
        window_rounding: spacing::RADIUS_MD.into(),
        window_shadow: egui::epaint::Shadow {
            color: colors::SHADOW_AMBIENT,
            offset: egui::vec2(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
        },
        window_fill: colors::BG_ELEVATED,
        window_stroke: egui::Stroke::new(1.0_f32, colors::BORDER),

        menu_rounding: spacing::RADIUS_SM.into(),
        panel_fill: colors::BG_BASE,

        // Popup / tooltip
        popup_shadow: egui::epaint::Shadow {
            color: colors::SHADOW_KEY,
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
        },

        // Resize handle
        resize_corner_size: 16.0,

        // Clip rectangles
        clip_rect_margin: 0.0,

        ..Default::default()
    };

    // ─── Spacing ─────────────────────────────────────────────────────
    style.spacing = egui::style::Spacing {
        item_spacing: egui::vec2(spacing::ITEM_SPACING_X, spacing::ITEM_SPACING_Y),
        window_margin: egui::Margin::same(spacing::MD),
        button_padding: egui::vec2(spacing::BUTTON_PADDING_X, spacing::BUTTON_PADDING_Y),
        menu_margin: egui::Margin::same(spacing::SM),
        indent: spacing::LG,
        interact_size: egui::vec2(32.0, 32.0), // Minimum touch target
        slider_width: 120.0,
        text_edit_width: 200.0,
        combo_width: 160.0,
        tooltip_width: 300.0,
        scroll: egui::style::ScrollStyle {
            bar_width: 8.0,
            bar_inner_margin: 4.0,
            bar_outer_margin: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };

    // ─── Typography ──────────────────────────────────────────────────
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(22.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
        (egui::TextStyle::Button, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(11.0, egui::FontFamily::Proportional)),
    ].into();

    // ─── Animation ───────────────────────────────────────────────────
    // Note: Animation config moved/removed in newer egui versions
    // style.animation = egui::Animation { ... };

    style
}

/// Unified widget visuals builder for consistency
fn widget_visuals(bg_fill: egui::Color32, fg_color: egui::Color32, stroke_color: egui::Color32) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill,
        weak_bg_fill: bg_fill,
        bg_stroke: egui::Stroke::new(1.0_f32, stroke_color),
        fg_stroke: egui::Stroke::new(1.0_f32, fg_color),
        rounding: spacing::RADIUS_SM.into(),
        expansion: 0.0,
    }
}

/// Setup custom fonts (JetBrains Mono for monospace, system UI for proportional)
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try to load JetBrains Mono from embedded assets if available
    // Note: bundled_fonts feature not defined, so this is skipped
    // If you add bundled_fonts feature to Cargo.toml, uncomment below:
    // fonts.font_data.insert(
    //     "jetbrains_mono".to_owned(),
    //     egui::FontData::from_static(include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf")),
    // );
    // fonts.families.entry(egui::FontFamily::Monospace).or_default()
    //     .insert(0, "jetbrains_mono".to_owned());

    // Use system UI font for proportional (respects OS font scaling)
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(0, "system-ui".to_owned());

    ctx.set_fonts(fonts);
}

/// Helper: Get a style for a floating/tool window (toolbar, popovers)
pub fn floating_window_style() -> egui::Style {
    let mut style = modern_dark_theme();
    style.visuals.window_shadow = egui::epaint::Shadow {
        color: colors::SHADOW_KEY,
        offset: egui::vec2(0.0, 12.0),
        blur: 32.0,
        spread: 4.0,
    };
    style.visuals.window_fill = colors::BG_ELEVATED;
    style.visuals.window_rounding = spacing::RADIUS_LG.into();
    style.visuals.window_stroke = egui::Stroke::new(1.0_f32, colors::BORDER_STRONG);
    style
}

/// Helper: Get minimal button visuals (icon-only, toolbar style)
pub fn minimal_button_visuals() -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: egui::Color32::TRANSPARENT,
        weak_bg_fill: egui::Color32::TRANSPARENT,
        bg_stroke: egui::Stroke::NONE,
        fg_stroke: egui::Stroke::new(1.0_f32, colors::TEXT_SECONDARY),
        rounding: spacing::RADIUS_SM.into(),
        expansion: 0.0,
    }
}

/// Helper: Get accent button visuals (primary actions)
pub fn accent_button_visuals() -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: colors::ACCENT,
        weak_bg_fill: colors::ACCENT_MUTED,
        bg_stroke: egui::Stroke::NONE,
        fg_stroke: egui::Stroke::new(1.0_f32, colors::TEXT_INVERSE),
        rounding: spacing::RADIUS_MD.into(),
        expansion: 0.0,
    }
}

/// Pixel art palette presets (kept for user convenience)
pub mod palette {
    use egui::Color32;

    pub const PICO8: &[Color32] = &[
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(29, 43, 83),
        Color32::from_rgb(126, 37, 83),
        Color32::from_rgb(0, 135, 81),
        Color32::from_rgb(171, 82, 54),
        Color32::from_rgb(95, 87, 79),
        Color32::from_rgb(194, 195, 199),
        Color32::from_rgb(255, 241, 232),
        Color32::from_rgb(255, 0, 77),
        Color32::from_rgb(255, 163, 0),
        Color32::from_rgb(255, 236, 39),
        Color32::from_rgb(0, 228, 54),
        Color32::from_rgb(41, 173, 255),
        Color32::from_rgb(131, 118, 156),
        Color32::from_rgb(255, 119, 168),
        Color32::from_rgb(255, 204, 170),
    ];

    pub const DB16: &[Color32] = &[
        Color32::from_rgb(20, 12, 28),
        Color32::from_rgb(68, 36, 52),
        Color32::from_rgb(48, 52, 109),
        Color32::from_rgb(78, 74, 78),
        Color32::from_rgb(133, 76, 48),
        Color32::from_rgb(52, 101, 36),
        Color32::from_rgb(208, 70, 72),
        Color32::from_rgb(117, 113, 97),
        Color32::from_rgb(89, 125, 206),
        Color32::from_rgb(210, 125, 44),
        Color32::from_rgb(133, 149, 161),
        Color32::from_rgb(109, 170, 44),
        Color32::from_rgb(210, 170, 153),
        Color32::from_rgb(109, 194, 202),
        Color32::from_rgb(218, 212, 94),
        Color32::from_rgb(222, 238, 214),
    ];

    pub const DB32: &[Color32] = &[
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(34, 32, 52),
        Color32::from_rgb(69, 40, 60),
        Color32::from_rgb(102, 57, 49),
        Color32::from_rgb(143, 86, 59),
        Color32::from_rgb(223, 113, 38),
        Color32::from_rgb(217, 160, 102),
        Color32::from_rgb(238, 195, 154),
        Color32::from_rgb(251, 242, 54),
        Color32::from_rgb(153, 229, 80),
        Color32::from_rgb(106, 190, 48),
        Color32::from_rgb(55, 148, 110),
        Color32::from_rgb(75, 105, 47),
        Color32::from_rgb(82, 75, 36),
        Color32::from_rgb(50, 60, 57),
        Color32::from_rgb(63, 63, 116),
        Color32::from_rgb(48, 96, 130),
        Color32::from_rgb(91, 110, 225),
        Color32::from_rgb(99, 155, 255),
        Color32::from_rgb(95, 205, 228),
        Color32::from_rgb(203, 219, 252),
        Color32::from_rgb(255, 255, 255),
        Color32::from_rgb(255, 175, 209),
        Color32::from_rgb(255, 134, 158),
        Color32::from_rgb(255, 94, 111),
        Color32::from_rgb(255, 56, 68),
        Color32::from_rgb(173, 35, 35),
        Color32::from_rgb(118, 66, 138),
        Color32::from_rgb(172, 50, 50),
        Color32::from_rgb(239, 72, 54),
        Color32::from_rgb(255, 108, 29),
        Color32::from_rgb(143, 39, 33),
    ];
}