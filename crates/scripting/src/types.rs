//! OverSprite Scripting - Types Module
//!
//! Lua type definitions and conversions for the scripting API.

use mlua::{Lua, Result as LuaResult, Table, Value, UserData, UserDataMethods};
use oversprite_core::{
    Rgba, Point, Size, Rect, ColorMode, BlendMode, LayerType, CelType,
    TileData, AnimationDirection, SliceKey, PaletteColor,
    Uuid, Fixed16_16
};

// ============================================================================
// Type conversion helpers
// ============================================================================

/// Convert Rgba to Lua table
pub fn rgba_to_lua(lua: &Lua, color: Rgba) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("r", color.r)?;
    table.set("g", color.g)?;
    table.set("b", color.b)?;
    table.set("a", color.a)?;
    Ok(table)
}

/// Convert Lua table to Rgba
pub fn lua_to_rgba(table: &Table) -> LuaResult<Rgba> {
    Ok(Rgba::new(
        table.get("r")?,
        table.get("g")?,
        table.get("b")?,
        table.get("a")?,
    ))
}

/// Convert Point to Lua table
pub fn point_to_lua(lua: &Lua, point: Point) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("x", point.x)?;
    table.set("y", point.y)?;
    Ok(table)
}

/// Convert Lua table to Point
pub fn lua_to_point(table: &Table) -> LuaResult<Point> {
    Ok(Point::new(table.get("x")?, table.get("y")?))
}

/// Convert Size to Lua table
pub fn size_to_lua(lua: &Lua, size: Size) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("w", size.w)?;
    table.set("h", size.h)?;
    Ok(table)
}

/// Convert Lua table to Size
pub fn lua_to_size(table: &Table) -> LuaResult<Size> {
    Ok(Size::new(table.get("w")?, table.get("h")?))
}

/// Convert Rect to Lua table
pub fn rect_to_lua(lua: &Lua, rect: Rect) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("x", rect.x)?;
    table.set("y", rect.y)?;
    table.set("w", rect.w)?;
    table.set("h", rect.h)?;
    Ok(table)
}

/// Convert Lua table to Rect
pub fn lua_to_rect(table: &Table) -> LuaResult<Rect> {
    Ok(Rect::new(
        table.get("x")?,
        table.get("y")?,
        table.get("w")?,
        table.get("h")?,
    ))
}

/// Convert Uuid to string
pub fn uuid_to_string(uuid: Uuid) -> String {
    uuid.to_string()
}

/// Convert string to Uuid
pub fn string_to_uuid(s: &str) -> Option<Uuid> {
    Uuid::parse_str(s).ok()
}

/// Convert TileData to Lua table
pub fn tile_data_to_lua(lua: &Lua, tile: TileData) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("tileId", tile.id)?;
    table.set("flipX", tile.flip_x)?;
    table.set("flipY", tile.flip_y)?;
    table.set("flipD", tile.flip_d)?;
    Ok(table)
}

/// Convert Lua table to TileData
pub fn lua_to_tile_data(table: &Table) -> LuaResult<TileData> {
    Ok(TileData {
        id: table.get("tileId")?,
        flip_x: table.get("flipX")?,
        flip_y: table.get("flipY")?,
        flip_d: table.get("flipD")?,
    })
}

/// Convert PaletteColor to Lua table
pub fn palette_color_to_lua(lua: &Lua, color: PaletteColor) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("r", color.r)?;
    table.set("g", color.g)?;
    table.set("b", color.b)?;
    table.set("a", color.a)?;
    table.set("name", color.name.unwrap_or_default())?;
    Ok(table)
}

/// Convert SliceKey to Lua table
pub fn slice_key_to_lua(lua: &Lua, key: SliceKey) -> LuaResult<Table<'_>> {
    let table = lua.create_table()?;
    table.set("frame", key.frame)?;
    table.set("x", key.bounds.x)?;
    table.set("y", key.bounds.y)?;
    table.set("w", key.bounds.w)?;
    table.set("h", key.bounds.h)?;
    if let Some(center) = key.center {
        table.set("centerX", center.x)?;
        table.set("centerY", center.y)?;
        table.set("centerW", center.w)?;
        table.set("centerH", center.h)?;
    }
    if let Some(pivot) = key.pivot {
        table.set("pivotX", pivot.x)?;
        table.set("pivotY", pivot.y)?;
    }
    table.set("hidden", key.hidden)?;
    Ok(table)
}

// ============================================================================
// Enum string conversions (using free functions since we can't impl on external types)
// ============================================================================

/// Convert BlendMode to Lua string
pub fn blend_mode_to_lua_string(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "normal",
        BlendMode::Multiply => "multiply",
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::Darken => "darken",
        BlendMode::Lighten => "lighten",
        BlendMode::ColorDodge => "colorDodge",
        BlendMode::ColorBurn => "colorBurn",
        BlendMode::HardLight => "hardLight",
        BlendMode::SoftLight => "softLight",
        BlendMode::Difference => "difference",
        BlendMode::Exclusion => "exclusion",
        BlendMode::Hue => "hue",
        BlendMode::Saturation => "saturation",
        BlendMode::Color => "color",
        BlendMode::Luminosity => "luminosity",
        BlendMode::Addition => "addition",
        BlendMode::Subtract => "subtract",
        BlendMode::Divide => "divide",
        BlendMode::Dissolve => "dissolve",
        BlendMode::LinearBurn => "linearBurn",
        BlendMode::LinearDodge => "linearDodge",
        BlendMode::DarkerColor => "darkerColor",
        BlendMode::LighterColor => "lighterColor",
        BlendMode::VividLight => "vividLight",
        BlendMode::LinearLight => "linearLight",
        BlendMode::PinLight => "pinLight",
        BlendMode::HardMix => "hardMix",
        BlendMode::Subtraction => "subtraction",
        BlendMode::AlphaDarken => "alphaDarken",
        BlendMode::SourceAtop => "sourceAtop",
    }
}

/// Convert Lua string to BlendMode
pub fn blend_mode_from_lua_string(s: &str) -> Option<BlendMode> {
    match s.to_lowercase().as_str() {
        "normal" => Some(BlendMode::Normal),
        "multiply" => Some(BlendMode::Multiply),
        "screen" => Some(BlendMode::Screen),
        "overlay" => Some(BlendMode::Overlay),
        "darken" => Some(BlendMode::Darken),
        "lighten" => Some(BlendMode::Lighten),
        "colordodge" => Some(BlendMode::ColorDodge),
        "colorburn" => Some(BlendMode::ColorBurn),
        "hardlight" => Some(BlendMode::HardLight),
        "softlight" => Some(BlendMode::SoftLight),
        "difference" => Some(BlendMode::Difference),
        "exclusion" => Some(BlendMode::Exclusion),
        "hue" => Some(BlendMode::Hue),
        "saturation" => Some(BlendMode::Saturation),
        "color" => Some(BlendMode::Color),
        "luminosity" => Some(BlendMode::Luminosity),
        "addition" => Some(BlendMode::Addition),
        "subtract" => Some(BlendMode::Subtract),
        "divide" => Some(BlendMode::Divide),
        "dissolve" => Some(BlendMode::Dissolve),
        "linearburn" => Some(BlendMode::LinearBurn),
        "lineardodge" => Some(BlendMode::LinearDodge),
        "darkercolor" => Some(BlendMode::DarkerColor),
        "lightercolor" => Some(BlendMode::LighterColor),
        "vividlight" => Some(BlendMode::VividLight),
        "linearlight" => Some(BlendMode::LinearLight),
        "pinlight" => Some(BlendMode::PinLight),
        "hardmix" => Some(BlendMode::HardMix),
        "subtraction" => Some(BlendMode::Subtraction),
        "alphadarken" => Some(BlendMode::AlphaDarken),
        "sourceatop" => Some(BlendMode::SourceAtop),
        _ => None,
    }
}

/// Convert LayerType to Lua string
pub fn layer_type_to_lua_string(lt: LayerType) -> &'static str {
    match lt {
        LayerType::Image => "image",
        LayerType::Group => "group",
        LayerType::Tilemap => "tilemap",
    }
}

/// Convert Lua string to LayerType
pub fn layer_type_from_lua_string(s: &str) -> Option<LayerType> {
    match s.to_lowercase().as_str() {
        "image" => Some(LayerType::Image),
        "group" => Some(LayerType::Group),
        "tilemap" => Some(LayerType::Tilemap),
        _ => None,
    }
}

/// Convert CelType to Lua string
pub fn cel_type_to_lua_string(ct: CelType) -> &'static str {
    match ct {
        CelType::Raw => "raw",
        CelType::Linked => "linked",
        CelType::Compressed => "compressed",
        CelType::Tilemap => "tilemap",
    }
}

/// Convert Lua string to CelType
pub fn cel_type_from_lua_string(s: &str) -> Option<CelType> {
    match s.to_lowercase().as_str() {
        "raw" => Some(CelType::Raw),
        "linked" => Some(CelType::Linked),
        "compressed" => Some(CelType::Compressed),
        "tilemap" => Some(CelType::Tilemap),
        _ => None,
    }
}

/// Convert ColorMode to Lua string
pub fn color_mode_to_lua_string(cm: ColorMode) -> &'static str {
    match cm {
        ColorMode::Rgba => "rgba",
        ColorMode::Grayscale => "grayscale",
        ColorMode::Indexed => "indexed",
        ColorMode::GrayscaleAlpha => "grayscaleAlpha",
    }
}

/// Convert Lua string to ColorMode
pub fn color_mode_from_lua_string(s: &str) -> Option<ColorMode> {
    match s.to_lowercase().as_str() {
        "rgba" => Some(ColorMode::Rgba),
        "grayscale" => Some(ColorMode::Grayscale),
        "indexed" => Some(ColorMode::Indexed),
        "grayscalealpha" => Some(ColorMode::GrayscaleAlpha),
        _ => None,
    }
}

/// Convert AnimationDirection to Lua string
pub fn animation_direction_to_lua_string(ad: AnimationDirection) -> &'static str {
    match ad {
        AnimationDirection::Forward => "forward",
        AnimationDirection::Reverse => "reverse",
        AnimationDirection::PingPong => "pingPong",
        AnimationDirection::PingPongReverse => "pingPongReverse",
    }
}

/// Convert Lua string to AnimationDirection
pub fn animation_direction_from_lua_string(s: &str) -> Option<AnimationDirection> {
    match s.to_lowercase().as_str() {
        "forward" => Some(AnimationDirection::Forward),
        "reverse" => Some(AnimationDirection::Reverse),
        "pingpong" => Some(AnimationDirection::PingPong),
        "pingpongreverse" => Some(AnimationDirection::PingPongReverse),
        _ => None,
    }
}

// ============================================================================
// UserData implementations for complex types
// ============================================================================

/// Lua wrapper for Uuid
#[derive(Clone, Copy)]
pub struct LuaUuid(pub Uuid);

impl UserData for LuaUuid {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("__tostring", |_, this, ()| Ok(this.0.to_string()));
        methods.add_method("__eq", |_, this, other: LuaUuid| Ok(this.0 == other.0));
        methods.add_method("bytes", |_, this, ()| Ok(this.0.as_bytes().to_vec()));
    }
}

/// Lua wrapper for Fixed16_16
#[derive(Clone, Copy)]
pub struct LuaFixed(pub Fixed16_16);

impl UserData for LuaFixed {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("__tostring", |_, this, ()| Ok(this.0.0.to_string()));
        methods.add_method("to_f32", |_, this, ()| Ok(this.0.0));
        methods.add_method("__add", |_, this, other: LuaFixed| Ok(LuaFixed(Fixed16_16(this.0.0 + other.0.0))));
        methods.add_method("__sub", |_, this, other: LuaFixed| Ok(LuaFixed(Fixed16_16(this.0.0 - other.0.0))));
        methods.add_method("__mul", |_, this, other: LuaFixed| Ok(LuaFixed(Fixed16_16(this.0.0 * other.0.0))));
        methods.add_method("__div", |_, this, other: LuaFixed| Ok(LuaFixed(Fixed16_16(this.0.0 / other.0.0))));
    }
}

/// Register all custom types with Lua
pub fn register_types(_lua: &Lua) -> LuaResult<()> {
    // No special registration needed for basic types
    // They are converted via the helper functions above
    Ok(())
}

// ============================================================================
// Type validation
// ============================================================================

/// Validate that a Lua value matches expected type
pub fn validate_type(value: &Value, expected: &str) -> LuaResult<()> {
    let type_name = match value {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "lightuserdata",
        Value::Integer(_) => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::UserData(_) => "userdata",
        Value::Error(_) => "error",
    };

    if type_name != expected {
        return Err(mlua::Error::FromLuaConversionError {
            from: type_name,
            to: "expected type",
            message: Some(format!("Expected {}, got {}", expected, type_name)),
        });
    }
    Ok(())
}

/// Get value with default
pub fn get_or_default<'lua, T>(table: &Table<'lua>, key: &str, default: T) -> LuaResult<T>
where
    T: mlua::FromLua<'lua>,
{
    match table.get::<&str, T>(key) {
        Ok(v) => Ok(v),
        Err(_) => Ok(default),
    }
}