//! Blend mode implementations matching Aseprite's blend modes

use crate::Rgba;

/// Blend mode enumeration matching Aseprite exactly (30 blend modes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    // Normal group
    Normal = 0,
    Dissolve = 1,

    // Darken group
    Darken = 2,
    Multiply = 3,
    ColorBurn = 4,
    LinearBurn = 5,
    DarkerColor = 6,

    // Lighten group
    Lighten = 7,
    Screen = 8,
    ColorDodge = 9,
    LinearDodge = 10,
    LighterColor = 11,

    // Contrast group
    Overlay = 12,
    SoftLight = 13,
    HardLight = 14,
    VividLight = 15,
    LinearLight = 16,
    PinLight = 17,
    HardMix = 18,

    // Inversion group
    Difference = 19,
    Exclusion = 20,
    Subtract = 21,
    Divide = 22,

    // Component group
    Hue = 23,
    Saturation = 24,
    Color = 25,
    Luminosity = 26,

    // Additional
    Addition = 27,
    Subtraction = 28,
    SourceAtop = 29,
    AlphaDarken = 30,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl BlendMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Dissolve),
            2 => Some(Self::Darken),
            3 => Some(Self::Multiply),
            4 => Some(Self::ColorBurn),
            5 => Some(Self::LinearBurn),
            6 => Some(Self::DarkerColor),
            7 => Some(Self::Lighten),
            8 => Some(Self::Screen),
            9 => Some(Self::ColorDodge),
            10 => Some(Self::LinearDodge),
            11 => Some(Self::LighterColor),
            12 => Some(Self::Overlay),
            13 => Some(Self::SoftLight),
            14 => Some(Self::HardLight),
            15 => Some(Self::VividLight),
            16 => Some(Self::LinearLight),
            17 => Some(Self::PinLight),
            18 => Some(Self::HardMix),
            19 => Some(Self::Difference),
            20 => Some(Self::Exclusion),
            21 => Some(Self::Subtract),
            22 => Some(Self::Divide),
            23 => Some(Self::Hue),
            24 => Some(Self::Saturation),
            25 => Some(Self::Color),
            26 => Some(Self::Luminosity),
            27 => Some(Self::Addition),
            28 => Some(Self::Subtraction),
            29 => Some(Self::SourceAtop),
            30 => Some(Self::AlphaDarken),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Dissolve => "Dissolve",
            Self::Darken => "Darken",
            Self::Multiply => "Multiply",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::DarkerColor => "Darker Color",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::ColorDodge => "Color Dodge",
            Self::LinearDodge => "Linear Dodge (Add)",
            Self::LighterColor => "Lighter Color",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft Light",
            Self::HardLight => "Hard Light",
            Self::VividLight => "Vivid Light",
            Self::LinearLight => "Linear Light",
            Self::PinLight => "Pin Light",
            Self::HardMix => "Hard Mix",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Subtract => "Subtract",
            Self::Divide => "Divide",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
            Self::Addition => "Addition",
            Self::Subtraction => "Subtraction",
            Self::SourceAtop => "Source Atop",
            Self::AlphaDarken => "Alpha Darken",
        }
    }

    /// Get all blend modes in display order
    pub fn all_modes() -> &'static [BlendMode] {
        &[
            BlendMode::Normal,
            BlendMode::Dissolve,
            BlendMode::Darken,
            BlendMode::Multiply,
            BlendMode::ColorBurn,
            BlendMode::LinearBurn,
            BlendMode::DarkerColor,
            BlendMode::Lighten,
            BlendMode::Screen,
            BlendMode::ColorDodge,
            BlendMode::LinearDodge,
            BlendMode::LighterColor,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::VividLight,
            BlendMode::LinearLight,
            BlendMode::PinLight,
            BlendMode::HardMix,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Subtract,
            BlendMode::Divide,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
            BlendMode::Addition,
            BlendMode::Subtraction,
            BlendMode::SourceAtop,
            BlendMode::AlphaDarken,
        ]
    }

    /// Get blend mode category for UI grouping
    pub fn category(&self) -> BlendCategory {
        match self {
            BlendMode::Normal | BlendMode::Dissolve => BlendCategory::Normal,
            BlendMode::Darken | BlendMode::Multiply | BlendMode::ColorBurn | BlendMode::LinearBurn | BlendMode::DarkerColor => BlendCategory::Darken,
            BlendMode::Lighten | BlendMode::Screen | BlendMode::ColorDodge | BlendMode::LinearDodge | BlendMode::LighterColor => BlendCategory::Lighten,
            BlendMode::Overlay | BlendMode::SoftLight | BlendMode::HardLight | BlendMode::VividLight | BlendMode::LinearLight | BlendMode::PinLight | BlendMode::HardMix => BlendCategory::Contrast,
            BlendMode::Difference | BlendMode::Exclusion | BlendMode::Subtract | BlendMode::Divide => BlendCategory::Inversion,
            BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => BlendCategory::Component,
            BlendMode::Addition | BlendMode::Subtraction | BlendMode::SourceAtop | BlendMode::AlphaDarken => BlendCategory::Additional,
        }
    }
}

/// Blend mode categories for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendCategory {
    Normal,
    Darken,
    Lighten,
    Contrast,
    Inversion,
    Component,
    Additional,
}

/// Apply blend mode to two RGBA colors (src over dst)
pub fn apply(mode: BlendMode, src: Rgba, dst: Rgba) -> Rgba {
    let src_f = to_f32(src);
    let dst_f = to_f32(dst);

    let result_f = match mode {
        BlendMode::Normal => normal(src_f, dst_f),
        BlendMode::Dissolve => normal(src_f, dst_f), // Approximation
        BlendMode::Darken => darken(src_f, dst_f),
        BlendMode::Multiply => multiply(src_f, dst_f),
        BlendMode::ColorBurn => color_burn(src_f, dst_f),
        BlendMode::LinearBurn => linear_burn(src_f, dst_f),
        BlendMode::DarkerColor => darker_color(src_f, dst_f),
        BlendMode::Lighten => lighten(src_f, dst_f),
        BlendMode::Screen => screen(src_f, dst_f),
        BlendMode::ColorDodge => color_dodge(src_f, dst_f),
        BlendMode::LinearDodge => linear_dodge(src_f, dst_f),
        BlendMode::LighterColor => lighter_color(src_f, dst_f),
        BlendMode::Overlay => overlay(src_f, dst_f),
        BlendMode::SoftLight => soft_light(src_f, dst_f),
        BlendMode::HardLight => hard_light(src_f, dst_f),
        BlendMode::VividLight => vivid_light(src_f, dst_f),
        BlendMode::LinearLight => linear_light(src_f, dst_f),
        BlendMode::PinLight => pin_light(src_f, dst_f),
        BlendMode::HardMix => hard_mix(src_f, dst_f),
        BlendMode::Difference => difference(src_f, dst_f),
        BlendMode::Exclusion => exclusion(src_f, dst_f),
        BlendMode::Subtract => subtract(src_f, dst_f),
        BlendMode::Divide => divide(src_f, dst_f),
        BlendMode::Hue => hue(src_f, dst_f),
        BlendMode::Saturation => saturation(src_f, dst_f),
        BlendMode::Color => color_blend(src_f, dst_f),
        BlendMode::Luminosity => luminosity(src_f, dst_f),
        BlendMode::Addition => addition(src_f, dst_f),
        BlendMode::Subtraction => subtraction(src_f, dst_f),
        BlendMode::SourceAtop => source_atop(src_f, dst_f),
        BlendMode::AlphaDarken => alpha_darken(src_f, dst_f),
    };

    // Apply alpha compositing (src over dst)
    let src_a = src_f[3];
    let dst_a = dst_f[3];
    let out_a = src_a + dst_a * (1.0 - src_a);

    if out_a == 0.0 {
        return Rgba::TRANSPARENT;
    }

    let out_r = (result_f[0] * src_a + dst_f[0] * dst_a * (1.0 - src_a)) / out_a;
    let out_g = (result_f[1] * src_a + dst_f[1] * dst_a * (1.0 - src_a)) / out_a;
    let out_b = (result_f[2] * src_a + dst_f[2] * dst_a * (1.0 - src_a)) / out_a;

    Rgba::new(
        (out_r * 255.0).round().clamp(0.0, 255.0) as u8,
        (out_g * 255.0).round().clamp(0.0, 255.0) as u8,
        (out_b * 255.0).round().clamp(0.0, 255.0) as u8,
        (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn to_f32(c: Rgba) -> [f32; 4] {
    [
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    ]
}

// Basic blend modes (RGB only, alpha handled separately)
fn normal(src: [f32; 4], _dst: [f32; 4]) -> [f32; 4] {
    [src[0], src[1], src[2], src[3]]
}

fn multiply(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [src[0] * dst[0], src[1] * dst[1], src[2] * dst[2], src[3]]
}

fn screen(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        1.0 - (1.0 - src[0]) * (1.0 - dst[0]),
        1.0 - (1.0 - src[1]) * (1.0 - dst[1]),
        1.0 - (1.0 - src[2]) * (1.0 - dst[2]),
        src[3],
    ]
}

fn overlay(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        if dst[0] < 0.5 { 2.0 * src[0] * dst[0] } else { 1.0 - 2.0 * (1.0 - src[0]) * (1.0 - dst[0]) },
        if dst[1] < 0.5 { 2.0 * src[1] * dst[1] } else { 1.0 - 2.0 * (1.0 - src[1]) * (1.0 - dst[1]) },
        if dst[2] < 0.5 { 2.0 * src[2] * dst[2] } else { 1.0 - 2.0 * (1.0 - src[2]) * (1.0 - dst[2]) },
        src[3],
    ]
}

fn darken(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [src[0].min(dst[0]), src[1].min(dst[1]), src[2].min(dst[2]), src[3]]
}

fn lighten(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [src[0].max(dst[0]), src[1].max(dst[1]), src[2].max(dst[2]), src[3]]
}

fn color_dodge(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        if src[0] == 1.0 { 1.0 } else { (dst[0] / (1.0 - src[0])).min(1.0) },
        if src[1] == 1.0 { 1.0 } else { (dst[1] / (1.0 - src[1])).min(1.0) },
        if src[2] == 1.0 { 1.0 } else { (dst[2] / (1.0 - src[2])).min(1.0) },
        src[3],
    ]
}

fn color_burn(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        if src[0] == 0.0 { 0.0 } else { 1.0 - (1.0 - dst[0]) / src[0] }.max(0.0),
        if src[1] == 0.0 { 0.0 } else { 1.0 - (1.0 - dst[1]) / src[1] }.max(0.0),
        if src[2] == 0.0 { 0.0 } else { 1.0 - (1.0 - dst[2]) / src[2] }.max(0.0),
        src[3],
    ]
}

fn linear_burn(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        (dst[0] + src[0] - 1.0).max(0.0),
        (dst[1] + src[1] - 1.0).max(0.0),
        (dst[2] + src[2] - 1.0).max(0.0),
        src[3],
    ]
}

fn darker_color(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let src_lum = src[0] + src[1] + src[2];
    let dst_lum = dst[0] + dst[1] + dst[2];
    if src_lum < dst_lum { src } else { dst }
}

fn linear_dodge(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        (src[0] + dst[0]).min(1.0),
        (src[1] + dst[1]).min(1.0),
        (src[2] + dst[2]).min(1.0),
        src[3],
    ]
}

fn lighter_color(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let src_lum = src[0] + src[1] + src[2];
    let dst_lum = dst[0] + dst[1] + dst[2];
    if src_lum > dst_lum { src } else { dst }
}

fn hard_light(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    // Hard light is overlay with src/dst swapped
    overlay(dst, src)
}

fn soft_light(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let mut out = [0.0; 4];
    for i in 0..3 {
        if src[i] < 0.5 {
            out[i] = dst[i] - (1.0 - 2.0 * src[i]) * dst[i] * (1.0 - dst[i]);
        } else {
            let d = if dst[i] < 0.25 {
                ((16.0 * dst[i] - 12.0) * dst[i] + 4.0) * dst[i]
            } else {
                dst[i].sqrt()
            };
            out[i] = dst[i] + (2.0 * src[i] - 1.0) * (d - dst[i]);
        }
    }
    out[3] = src[3];
    out
}

fn vivid_light(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    // Vivid Light: Color Dodge if src > 0.5, Color Burn if src < 0.5
    let mut out = [0.0; 4];
    for i in 0..3 {
        out[i] = if src[i] < 0.5 {
            // Color Burn
            if 2.0 * src[i] == 0.0 { 0.0 } else { 1.0 - (1.0 - dst[i]) / (2.0 * src[i]) }.max(0.0)
        } else {
            // Color Dodge
            if 2.0 * src[i] - 1.0 == 1.0 { 1.0 } else { dst[i] / (1.0 - (2.0 * src[i] - 1.0)) }.min(1.0)
        };
    }
    out[3] = src[3];
    out
}

fn linear_light(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    // Linear Light: Linear Dodge if src > 0.5, Linear Burn if src < 0.5
    let mut out = [0.0; 4];
    for i in 0..3 {
        out[i] = if src[i] < 0.5 {
            // Linear Burn
            (dst[i] + 2.0 * src[i] - 1.0).max(0.0)
        } else {
            // Linear Dodge
            (dst[i] + 2.0 * src[i] - 1.0).min(1.0)
        };
    }
    out[3] = src[3];
    out
}

fn pin_light(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    // Pin Light: Lighten if src > 0.5, Darken if src < 0.5
    let mut out = [0.0; 4];
    for i in 0..3 {
        out[i] = if src[i] < 0.5 {
            dst[i].min(2.0 * src[i])
        } else {
            dst[i].max(2.0 * src[i] - 1.0)
        };
    }
    out[3] = src[3];
    out
}

fn hard_mix(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    // Hard Mix: Posterize result of Vivid Light
    let mut out = vivid_light(src, dst);
    for i in 0..3 {
        out[i] = if out[i] < 0.5 { 0.0 } else { 1.0 };
    }
    out[3] = src[3];
    out
}

fn difference(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [(src[0] - dst[0]).abs(), (src[1] - dst[1]).abs(), (src[2] - dst[2]).abs(), src[3]]
}

fn exclusion(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        src[0] + dst[0] - 2.0 * src[0] * dst[0],
        src[1] + dst[1] - 2.0 * src[1] * dst[1],
        src[2] + dst[2] - 2.0 * src[2] * dst[2],
        src[3],
    ]
}

fn subtract(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        (dst[0] - src[0]).max(0.0),
        (dst[1] - src[1]).max(0.0),
        (dst[2] - src[2]).max(0.0),
        src[3],
    ]
}

fn division(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        if src[0] == 0.0 { 1.0 } else { (dst[0] / src[0]).min(1.0) },
        if src[1] == 0.0 { 1.0 } else { (dst[1] / src[1]).min(1.0) },
        if src[2] == 0.0 { 1.0 } else { (dst[2] / src[2]).min(1.0) },
        src[3],
    ]
}

// HSL-based blend modes (simplified)
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    let hue_to_rgb = |p: f32, q: f32, t: f32| {
        let t = if t < 0.0 { t + 1.0 } else if t > 1.0 { t - 1.0 } else { t };
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

fn hue(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let (h, _, _) = rgb_to_hsl(src[0], src[1], src[2]);
    let (_, s, l) = rgb_to_hsl(dst[0], dst[1], dst[2]);
    let (r, g, b) = hsl_to_rgb(h, s, l);
    [r, g, b, src[3]]
}

fn saturation(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let (_, s, _) = rgb_to_hsl(src[0], src[1], src[2]);
    let (h, _, l) = rgb_to_hsl(dst[0], dst[1], dst[2]);
    let (r, g, b) = hsl_to_rgb(h, s, l);
    [r, g, b, src[3]]
}

fn color_blend(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let (h, s, _) = rgb_to_hsl(src[0], src[1], src[2]);
    let (_, _, l) = rgb_to_hsl(dst[0], dst[1], dst[2]);
    let (r, g, b) = hsl_to_rgb(h, s, l);
    [r, g, b, src[3]]
}

fn luminosity(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let (_, _, l) = rgb_to_hsl(src[0], src[1], src[2]);
    let (h, s, _) = rgb_to_hsl(dst[0], dst[1], dst[2]);
    let (r, g, b) = hsl_to_rgb(h, s, l);
    [r, g, b, src[3]]
}

fn addition(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        (src[0] + dst[0]).min(1.0),
        (src[1] + dst[1]).min(1.0),
        (src[2] + dst[2]).min(1.0),
        src[3],
    ]
}

fn subtraction(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        (dst[0] - src[0]).max(0.0),
        (dst[1] - src[1]).max(0.0),
        (dst[2] - src[2]).max(0.0),
        src[3],
    ]
}

fn divide(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    division(src, dst)
}

fn source_atop(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    [
        src[0] * dst[3] + dst[0] * (1.0 - src[3]),
        src[1] * dst[3] + dst[1] * (1.0 - src[3]),
        src[2] * dst[3] + dst[2] * (1.0 - src[3]),
        src[3],
    ]
}

fn alpha_darken(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    // Alpha Darken: blend only where src has alpha
    [
        src[0] * src[3] + dst[0] * (1.0 - src[3]),
        src[1] * src[3] + dst[1] * (1.0 - src[3]),
        src[2] * src[3] + dst[2] * (1.0 - src[3]),
        src[3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rgba;

    #[test]
    fn test_normal_blend() {
        let src = Rgba::new(255, 0, 0, 255);
        let dst = Rgba::new(0, 255, 0, 255);
        let result = apply(BlendMode::Normal, src, dst);
        assert_eq!(result, src);
    }

    #[test]
    fn test_multiply_blend() {
        let src = Rgba::new(255, 0, 0, 255);
        let dst = Rgba::new(0, 255, 0, 255);
        let result = apply(BlendMode::Multiply, src, dst);
        assert_eq!(result.r, 0);
        assert_eq!(result.g, 0);
        assert_eq!(result.b, 0);
    }

    #[test]
    fn test_screen_blend() {
        let src = Rgba::new(255, 0, 0, 255);
        let dst = Rgba::new(0, 255, 0, 255);
        let result = apply(BlendMode::Screen, src, dst);
        assert_eq!(result.r, 255);
        assert_eq!(result.g, 255);
        assert_eq!(result.b, 0);
    }

    #[test]
    fn test_all_modes_count() {
        assert_eq!(BlendMode::all_modes().len(), 31);
    }
}