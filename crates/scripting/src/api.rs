//! OverSprite Scripting - API Module
//!
//! Lua API bindings for sprite, layer, frame, cel, image, and other core types.

use mlua::{Lua, Result as LuaResult, Value, UserData, UserDataMethods};
use oversprite_core::{Sprite, Layer, Frame, Cel, Palette, Rgba, ColorMode, LayerType, Tileset, Tag, Slice, Grid};
use std::sync::{Arc, Mutex};

// ============================================================================
// Sprite API
// ============================================================================

#[derive(Clone)]
pub struct SpriteApi {
    pub sprite: Arc<Mutex<Sprite>>,
}

impl UserData for SpriteApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("width", |_, this, ()| {
            Ok(this.sprite.lock().unwrap().metadata.dimensions.w)
        });
        methods.add_method("height", |_, this, ()| {
            Ok(this.sprite.lock().unwrap().metadata.dimensions.h)
        });
        methods.add_method("size", |_, this, ()| {
            let s = this.sprite.lock().unwrap();
            Ok(SizeApi { w: s.metadata.dimensions.w, h: s.metadata.dimensions.h })
        });
        methods.add_method("color_mode", |_, this, ()| {
            Ok(format!("{:?}", this.sprite.lock().unwrap().metadata.color_mode))
        });
        methods.add_method("frames", |_, this, ()| {
            Ok(this.sprite.lock().unwrap().frames.order.len())
        });
        methods.add_method("layers", |_, this, ()| {
            let sprite = this.sprite.lock().unwrap();
            let layers: Vec<LayerApi> = sprite.layers.order.iter()
                .filter_map(|uuid| sprite.layer_data.get(uuid).map(|l| LayerApi::new(l.clone())))
                .collect();
            Ok(layers)
        });
        methods.add_method("tags", |_, this, ()| {
            let sprite = this.sprite.lock().unwrap();
            let tags: Vec<TagApi> = sprite.tags.iter().map(|t| TagApi::new(t.clone())).collect();
            Ok(tags)
        });
        methods.add_method("slices", |_, this, ()| {
            let sprite = this.sprite.lock().unwrap();
            let slices: Vec<SliceApi> = sprite.slices.iter().map(|s| SliceApi::new(s.clone())).collect();
            Ok(slices)
        });
        methods.add_method("palette", |_, this, ()| {
            Ok(PaletteApi::new(this.sprite.lock().unwrap().global_palette.clone()))
        });
        methods.add_method("save_as", |_, this, path: String| {
            let sprite = this.sprite.lock().unwrap();
            oversprite_parser::write_sprite(&sprite, std::path::Path::new(&path))
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))
        });
        methods.add_method("close", |_, _this, ()| {
            // Mark sprite as closed
            Ok(())
        });
        methods.add_method("new_layer", |_, this, (name, layer_type): (String, Option<String>)| {
            let mut sprite = this.sprite.lock().unwrap();
            let lt = match layer_type.as_deref() {
                Some("group") => LayerType::Group,
                Some("tilemap") => LayerType::Tilemap,
                _ => LayerType::Image,
            };
            let mut layer = Layer::new_image(name);
            layer.layer_type = lt;
            sprite.add_layer(layer.clone(), None);
            Ok(LayerApi::new(layer))
        });
        methods.add_method("new_frame", |_, this, ()| {
            let mut sprite = this.sprite.lock().unwrap();
            let duration_ms = 100;
            let uuid = sprite.add_frame(duration_ms);
            let frame = sprite.frame_data.get(&uuid).cloned().unwrap_or_else(|| Frame::new(duration_ms));
            Ok(FrameApi::new(frame))
        });
    }
}

// ============================================================================
// Layer API
// ============================================================================

#[derive(Clone)]
pub struct LayerApi {
    pub layer: Layer,
}

impl LayerApi {
    pub fn new(layer: Layer) -> Self {
        Self { layer }
    }
}

impl UserData for LayerApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("name", |_, this, ()| Ok(this.layer.name.clone()));
        methods.add_method_mut("set_name", |_, this, name: String| {
            this.layer.name = name;
            Ok(())
        });
        methods.add_method("type", |_, this, ()| Ok(format!("{:?}", this.layer.layer_type)));
        methods.add_method("visible", |_, this, ()| Ok(this.layer.visible));
        methods.add_method_mut("set_visible", |_, this, v: bool| {
            this.layer.visible = v;
            Ok(())
        });
        methods.add_method("opacity", |_, this, ()| Ok(this.layer.opacity as i32));
        methods.add_method_mut("set_opacity", |_, this, v: i32| {
            this.layer.opacity = v.clamp(0, 255) as u8;
            Ok(())
        });
        methods.add_method("blend_mode", |_, this, ()| Ok(format!("{:?}", this.layer.blend_mode)));
        methods.add_method_mut("set_blend_mode", |_, this, mode: String| {
            if let Some(bm) = crate::types::blend_mode_from_lua_string(&mode) {
                this.layer.blend_mode = bm;
            }
            Ok(())
        });
        methods.add_method("z_index", |_, this, ()| Ok(this.layer.z_index as i32));
        methods.add_method_mut("set_z_index", |_, this, v: i32| {
            this.layer.z_index = v as i16;
            Ok(())
        });
        methods.add_method("is_group", |_, this, ()| Ok(this.layer.layer_type == LayerType::Group));
        methods.add_method("is_tilemap", |_, this, ()| Ok(this.layer.layer_type == LayerType::Tilemap));
        methods.add_method("delete", |_, _this, ()| {
            // Would need sprite reference to actually delete
            Ok(())
        });
    }
}

// ============================================================================
// Frame API
// ============================================================================

#[derive(Clone)]
pub struct FrameApi {
    pub frame: Frame,
}

impl FrameApi {
    pub fn new(frame: Frame) -> Self {
        Self { frame }
    }
}

impl UserData for FrameApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("duration", |_, this, ()| Ok(this.frame.duration_ms as i32));
        methods.add_method_mut("set_duration", |_, this, ms: i32| {
            this.frame.duration_ms = ms.max(1) as u32;
            Ok(())
        });
        methods.add_method("cels", |_, _this, ()| {
            // Would need sprite reference to get cels
            Ok(Vec::<CelApi>::new())
        });
        methods.add_method("number", |_, _this, ()| Ok(0)); // Would need index
    }
}

// ============================================================================
// Cel API
// ============================================================================

#[derive(Clone)]
pub struct CelApi {
    pub cel: Cel,
}

impl CelApi {
    pub fn new(cel: Cel) -> Self {
        Self { cel }
    }
}

impl UserData for CelApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("position", |_, this, ()| {
            Ok(PointApi { x: this.cel.position.x, y: this.cel.position.y })
        });
        methods.add_method_mut("set_position", |_, this, pos: PointApi| {
            this.cel.position.x = pos.x;
            this.cel.position.y = pos.y;
            Ok(())
        });
        methods.add_method("opacity", |_, this, ()| Ok(this.cel.opacity as i32));
        methods.add_method_mut("set_opacity", |_, this, v: i32| {
            this.cel.opacity = v.clamp(0, 255) as u8;
            Ok(())
        });
        methods.add_method("bounds", |_, this, ()| {
            Ok(RectApi {
                x: this.cel.bounds.x,
                y: this.cel.bounds.y,
                w: this.cel.bounds.w,
                h: this.cel.bounds.h,
            })
        });
        methods.add_method("type", |_, this, ()| Ok(format!("{:?}", this.cel.cel_type)));
        methods.add_method("image", |_, _this, ()| {
            // Return image if available
            Ok(None::<ImageApi>)
        });
    }
}

// ============================================================================
// Image API
// ============================================================================

#[derive(Clone)]
pub struct ImageApi {
    pub width: u32,
    pub height: u32,
    pub color_mode: ColorMode,
    pixels: Vec<u8>,
}

impl ImageApi {
    pub fn new(width: u32, height: u32, color_mode: ColorMode) -> Self {
        let bytes_per_pixel = match color_mode {
            ColorMode::Rgba => 4,
            ColorMode::Grayscale => 1,
            ColorMode::Indexed => 1,
            ColorMode::GrayscaleAlpha => 2,
        };
        Self {
            width,
            height,
            color_mode,
            pixels: vec![0; (width * height * bytes_per_pixel) as usize],
        }
    }

    pub fn from_bytes(bytes: &[u8], width: u32, height: u32, color_mode: ColorMode) -> Self {
        let mut img = Self::new(width, height, color_mode);
        img.pixels = bytes.to_vec();
        img
    }
}

impl UserData for ImageApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("width", |_, this, ()| Ok(this.width as i32));
        methods.add_method("height", |_, this, ()| Ok(this.height as i32));
        methods.add_method("draw_pixel", |_, this, (x, y, color): (i32, i32, ColorApi)| {
            if x >= 0 && x < this.width as i32 && y >= 0 && y < this.height as i32 {
                let _rgba = Rgba::new(color.r, color.g, color.b, color.a);
                // Draw pixel logic
            }
            Ok(())
        });
        methods.add_method("get_pixel", |_, this, (x, y): (i32, i32)| {
            if x >= 0 && x < this.width as i32 && y >= 0 && y < this.height as i32 {
                // Return pixel color
                Ok(Some(ColorApi { r: 255, g: 255, b: 255, a: 255 }))
            } else {
                Ok(None)
            }
        });
        methods.add_method("clear", |_, _this, color: Option<ColorApi>| {
            let _c = color.map(|c| Rgba::new(c.r, c.g, c.b, c.a)).unwrap_or(Rgba::TRANSPARENT);
            // Clear image
            Ok(())
        });
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));
    }
}

// ============================================================================
// Color API
// ============================================================================

#[derive(Clone, Copy)]
pub struct ColorApi {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl UserData for ColorApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("__tostring", |_, this, ()| {
            Ok(format!("rgba({}, {}, {}, {})", this.r, this.g, this.b, this.a))
        });
    }
}

// ============================================================================
// Point API
// ============================================================================

#[derive(Clone, Copy)]
pub struct PointApi {
    pub x: i32,
    pub y: i32,
}

impl UserData for PointApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("__add", |_, this, other: PointApi| {
            Ok(PointApi { x: this.x + other.x, y: this.y + other.y })
        });
        methods.add_method("__sub", |_, this, other: PointApi| {
            Ok(PointApi { x: this.x - other.x, y: this.y - other.y })
        });
    }
}

// ============================================================================
// Size API
// ============================================================================

#[derive(Clone, Copy)]
pub struct SizeApi {
    pub w: u32,
    pub h: u32,
}

impl UserData for SizeApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

// ============================================================================
// Rect API
// ============================================================================

#[derive(Clone, Copy)]
pub struct RectApi {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl UserData for RectApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("contains", |_, this, pt: PointApi| {
            Ok(pt.x >= this.x && pt.x < this.x + this.w as i32 &&
               pt.y >= this.y && pt.y < this.y + this.h as i32)
        });
    }
}

// ============================================================================
// Palette API
// ============================================================================

#[derive(Clone)]
pub struct PaletteApi {
    pub palette: Palette,
}

impl PaletteApi {
    pub fn new(palette: Palette) -> Self {
        Self { palette }
    }
}

impl UserData for PaletteApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("size", |_, this, ()| Ok(this.palette.colors.len() as i32));
        methods.add_method("get_color", |_, this, index: i32| {
            if index >= 0 && index < this.palette.colors.len() as i32 {
                let c = &this.palette.colors[index as usize];
                Ok(Some(ColorApi { r: c.r, g: c.g, b: c.b, a: c.a }))
            } else {
                Ok(None)
            }
        });
        methods.add_method_mut("set_color", |_, this, (index, color): (i32, ColorApi)| {
            if index >= 0 && index < this.palette.colors.len() as i32 {
                this.palette.colors[index as usize] = oversprite_core::PaletteColor {
                    r: color.r, g: color.g, b: color.b, a: color.a, name: None
                };
            }
            Ok(())
        });
    }
}

// ============================================================================
// Tag API
// ============================================================================

#[derive(Clone)]
pub struct TagApi {
    pub tag: Tag,
}

impl TagApi {
    pub fn new(tag: Tag) -> Self {
        Self { tag }
    }
}

impl UserData for TagApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("name", |_, this, ()| Ok(this.tag.name.clone()));
        methods.add_method("from_frame", |_, this, ()| Ok(this.tag.from_frame as i32));
        methods.add_method("to_frame", |_, this, ()| Ok(this.tag.to_frame as i32));
        methods.add_method("direction", |_, this, ()| Ok(format!("{:?}", this.tag.direction)));
    }
}

// ============================================================================
// Slice API
// ============================================================================

#[derive(Clone)]
pub struct SliceApi {
    pub slice: Slice,
}

impl SliceApi {
    pub fn new(slice: Slice) -> Self {
        Self { slice }
    }
}

impl UserData for SliceApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("name", |_, this, ()| Ok(this.slice.name.clone()));
        methods.add_method("bounds", |_, this, ()| {
            if let Some(key) = this.slice.keys.first() {
                Ok(Some(RectApi {
                    x: key.bounds.x,
                    y: key.bounds.y,
                    w: key.bounds.w,
                    h: key.bounds.h,
                }))
            } else { Ok(None) }
        });
    }
}

// ============================================================================
// Tileset API
// ============================================================================

#[derive(Clone)]
pub struct TilesetApi {
    pub tileset: Tileset,
}

impl UserData for TilesetApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("tile_width", |_, this, ()| Ok(this.tileset.tile_width as i32));
        methods.add_method("tile_height", |_, this, ()| Ok(this.tileset.tile_height as i32));
        methods.add_method("tile_count", |_, this, ()| Ok(this.tileset.tile_count as i32));
    }
}

// ============================================================================
// TileData API
// ============================================================================

#[derive(Clone, Copy)]
pub struct TileDataApi {
    pub id: u32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_d: bool,
}

impl UserData for TileDataApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

// ============================================================================
// Grid API
// ============================================================================

#[derive(Clone)]
pub struct GridApi {
    pub grid: Grid,
}

impl UserData for GridApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

// ============================================================================
// App API (global)
// ============================================================================

pub struct AppApi {
    pub active_sprite: Option<Arc<Mutex<Sprite>>>,
    pub fg_color: Rgba,
    pub bg_color: Rgba,
}

impl AppApi {
    /// Create an app handle with no active sprite.
    pub fn new() -> Self {
        Self {
            active_sprite: None,
            fg_color: Rgba::BLACK,
            bg_color: Rgba::WHITE,
        }
    }

    /// Create an app handle bound to a sprite.
    pub fn with_sprite(sprite: Arc<Mutex<Sprite>>) -> Self {
        Self {
            active_sprite: Some(sprite),
            fg_color: Rgba::BLACK,
            bg_color: Rgba::WHITE,
        }
    }
}

impl Default for AppApi {
    fn default() -> Self {
        Self::new()
    }
}

impl UserData for AppApi {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("active_sprite", |_, this, ()| {
            Ok(this.active_sprite.as_ref().map(|s| SpriteApi { sprite: s.clone() }))
        });
        methods.add_method("fg_color", |_, this, ()| Ok(ColorApi::from(this.fg_color)));
        methods.add_method_mut("set_fg_color", |_, this, color: ColorApi| {
            this.fg_color = Rgba::new(color.r, color.g, color.b, color.a);
            Ok(())
        });
        methods.add_method("bg_color", |_, this, ()| Ok(ColorApi::from(this.bg_color)));
        methods.add_method_mut("set_bg_color", |_, this, color: ColorApi| {
            this.bg_color = Rgba::new(color.r, color.g, color.b, color.a);
            Ok(())
        });
        methods.add_method("refresh", |_, _, ()| Ok(()));
        methods.add_method("alert", |_, _, msg: String| {
            eprintln!("Alert: {}", msg);
            Ok(())
        });
    }
}

impl From<Rgba> for ColorApi {
    fn from(c: Rgba) -> Self {
        Self { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}

// ============================================================================
// Trait implementations for mlua
// ============================================================================

impl<'lua> mlua::FromLuaMulti<'lua> for PointApi {
    fn from_lua_multi(values: mlua::MultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let mut iter = values.into_iter();
        let x = <i32 as mlua::FromLua>::from_lua(iter.next().unwrap_or(Value::Nil), lua)?;
        let y = <i32 as mlua::FromLua>::from_lua(iter.next().unwrap_or(Value::Nil), lua)?;
        Ok(PointApi { x, y })
    }
}

impl<'lua> mlua::FromLua<'lua> for ColorApi {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        if let Value::Table(table) = value {
            // Named fields: { r = .., g = .., b = .., a = .. }
            let r = table.get("r")?;
            let g = table.get("g")?;
            let b = table.get("b")?;
            let a = table.get("a").unwrap_or(255);
            Ok(ColorApi { r, g, b, a })
        } else if let Value::Integer(v) = value {
            // Packed 0xRRGGBBAA integer
            let v = v as u32;
            Ok(ColorApi {
                r: ((v >> 24) & 0xFF) as u8,
                g: ((v >> 16) & 0xFF) as u8,
                b: ((v >> 8) & 0xFF) as u8,
                a: (v & 0xFF) as u8,
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "ColorApi",
                message: None,
            })
        }
    }
}

// `ColorApi` uses the blanket `FromLuaMulti` impl that mlua provides for every
// `T: FromLua`, so no manual `FromLuaMulti` impl is needed (and one would
// conflict with it).

impl<'lua> mlua::FromLuaMulti<'lua> for crate::types::LuaUuid {
    fn from_lua_multi(values: mlua::MultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let mut iter = values.into_iter();
        let s = <String as mlua::FromLua>::from_lua(iter.next().unwrap_or(Value::Nil), lua)?;
        Ok(crate::types::LuaUuid(crate::types::string_to_uuid(&s).unwrap_or_default()))
    }
}

impl<'lua> mlua::FromLuaMulti<'lua> for crate::types::LuaFixed {
    fn from_lua_multi(values: mlua::MultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let mut iter = values.into_iter();
        let f = <f32 as mlua::FromLua>::from_lua(iter.next().unwrap_or(Value::Nil), lua)?;
        Ok(crate::types::LuaFixed(oversprite_core::Fixed16_16(f)))
    }
}

// ============================================================================
// Register all APIs with Lua
// ============================================================================

pub fn register_apis(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    // App API
    let app = AppApi {
        active_sprite: None,
        fg_color: Rgba::BLACK,
        bg_color: Rgba::WHITE,
    };
    globals.set("app", app)?;

    // Constructors
    globals.set("Sprite", lua.create_function(|_lua, (w, h, mode): (u32, u32, String)| {
        let cm = match mode.as_str() {
            "rgba" => ColorMode::Rgba,
            "indexed" => ColorMode::Indexed,
            "grayscale" => ColorMode::Grayscale,
            _ => ColorMode::Rgba,
        };
        let sprite = Sprite::new_with_size(w, h, cm);
        Ok(SpriteApi { sprite: Arc::new(Mutex::new(sprite)) })
    })?)?;

    globals.set("Image", lua.create_function(|_, (w, h, mode): (u32, u32, String)| {
        let cm = match mode.as_str() {
            "rgba" => ColorMode::Rgba,
            "indexed" => ColorMode::Indexed,
            "grayscale" => ColorMode::Grayscale,
            _ => ColorMode::Rgba,
        };
        Ok(ImageApi::new(w, h, cm))
    })?)?;

    globals.set("Color", lua.create_function(|_, (r, g, b, a): (u8, u8, u8, Option<u8>)| {
        Ok(ColorApi { r, g, b, a: a.unwrap_or(255) })
    })?)?;

    globals.set("Point", lua.create_function(|_, (x, y): (i32, i32)| {
        Ok(PointApi { x, y })
    })?)?;

    globals.set("Rect", lua.create_function(|_, (x, y, w, h): (i32, i32, u32, u32)| {
        Ok(RectApi { x, y, w, h })
    })?)?;

    globals.set("Size", lua.create_function(|_, (w, h): (u32, u32)| {
        Ok(SizeApi { w, h })
    })?)?;

    // Dialog API
    globals.set("Dialog", lua.create_function(|lua, title: String| {
        let dialog = lua.create_table()?;
        dialog.set("title", title)?;
        dialog.set("show", lua.create_function(|_, ()| Ok(()))?)?;
        dialog.set("close", lua.create_function(|_, ()| Ok(()))?)?;
        Ok(dialog)
    })?)?;

    // Events
    let events = lua.create_table()?;
    events.set("on", lua.create_function(|_, (_event, _fn): (String, mlua::Function)| Ok(()))?)?;
    events.set("off", lua.create_function(|_, (_event, _fn): (String, mlua::Function)| Ok(()))?)?;
    globals.set("events", events)?;

    Ok(())
}