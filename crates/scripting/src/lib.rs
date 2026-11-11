//! OverSprite Scripting
//!
//! Lua scripting engine using mlua for extensions, automation,
//! and custom tool development. Compatible with Aseprite's scripting API.

pub mod api;
pub mod extensions;
pub mod events;
pub mod types;
pub mod context;

use mlua::{Lua, Result as LuaResult, Table, Value, Function};
use oversprite_core::{CoreError, CoreResult, Sprite, Point, Rgba, ColorMode};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Scripting engine state
pub struct ScriptingEngine {
    lua: Lua,
    extensions: HashMap<String, Extension>,
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: String,
    pub api_version: String,
}

impl Default for ScriptingEngine {
    fn default() -> Self {
        let lua = Lua::new();
        Self::init_stdlib(&lua).expect("Failed to initialize stdlib");
        Self {
            lua,
            extensions: HashMap::new(),
        }
    }
}

impl ScriptingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn init_stdlib(lua: &Lua) -> LuaResult<()> {
        // Load standard libraries
        lua.load(r#"
            -- OverSprite scripting API
            oversprite = {}
            oversprite.version = "1.0.0"

            -- Core types
            function oversprite.Point(x, y) return {x = x, y = y} end
            function oversprite.Size(w, h) return {width = w, height = h} end
            function oversprite.Rect(x, y, w, h) return {x = x, y = y, width = w, height = h} end
            function oversprite.Color(r, g, b, a) return {r = r, g = g, b = b, a = a or 255} end

            -- App API
            app = {}
            app.activeSprite = nil
            app.activeLayer = nil
            app.activeFrame = 1
            app.fgColor = {r = 0, g = 0, b = 0, a = 255}
            app.bgColor = {r = 255, g = 255, b = 255, a = 255}
            app.zoom = 1.0

            function app.refresh() end
            function app.undo() end
            function app.redo() end
            function app.alert(msg) print("ALERT: " .. msg) end

            -- Sprite API
            Sprite = {}
            function Sprite:new(width, height, colorMode) end
            function Sprite:saveAs(filename) end
            function Sprite:close() end

            -- Layer API
            Layer = {}
            function Layer:new(name, type) end
            function Layer:delete() end
            function Layer:isVisible() return true end
            function Layer:isEditable() return true end

            -- Frame API
            Frame = {}

            -- Cel API
            Cel = {}

            -- Image API
            Image = {}
            function Image:new(spec) end
            function Image:drawPixel(x, y, color) end
            function Image:getPixel(x, y) end

            -- Selection API
            Selection = {}
            function Selection:isEmpty() return true end
            function Selection:bounds() return nil end

            -- Tilemap API
            Tilemap = {}
            function Tilemap:new(tileset) end
            function Tilemap:setTile(x, y, tileId, flipX, flipY, flipD) end

            -- Tileset API
            Tileset = {}
            function Tileset:new(spec) end

            -- Tag API
            Tag = {}

            -- Slice API
            Slice = {}

            -- Dialog API
            Dialog = {}
            function Dialog:new(title) return {} end
            function Dialog:show() end
            function Dialog:close() end

            -- Command API
            Command = {}
            function Command:register(id, fn) end

            -- Events
            events = {}
            function events.on(event, fn) end
            function events.off(event, fn) end
        "#).exec()
    }

    /// Run a script string
    pub fn run_script(&mut self, script: &str) -> LuaResult<Value<'_>> {
        self.lua.load(script).eval()
    }

    /// Run a script file
    pub fn run_file(&mut self, path: &std::path::Path) -> LuaResult<Value<'_>> {
        let script = std::fs::read_to_string(path)?;
        self.run_script(&script)
    }

    /// Register an extension
    pub fn register_extension(&mut self, ext: Extension) -> CoreResult<()> {
        self.extensions.insert(ext.name.clone(), ext);
        Ok(())
    }

    /// Load an extension from a directory
    pub fn load_extension(&mut self, path: &std::path::Path) -> CoreResult<()> {
        let package_json = path.join("package.json");
        if !package_json.exists() {
            return Err(CoreError::Internal("package.json not found".into()));
        }

        let content = std::fs::read_to_string(package_json)?;
        let pkg: serde_json::Value = serde_json::from_str(&content)?;

        let ext = Extension {
            name: pkg.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            version: pkg.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string(),
            author: pkg.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: pkg.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            entry_point: pkg.get("main").and_then(|v| v.as_str()).unwrap_or("main.lua").to_string(),
            api_version: pkg.get("apiVersion").and_then(|v| v.as_str()).unwrap_or("1.0").to_string(),
        };

        let entry_path = path.join(&ext.entry_point);
        if entry_path.exists() {
            self.run_file(&entry_path)
                .map_err(|e| CoreError::Internal(e.to_string()))?;
        }

        self.register_extension(ext)
    }

    /// Get or create a global table
    pub fn globals(&mut self) -> Table<'_> {
        self.lua.globals()
    }

    /// Set a global value
    pub fn set_global(&mut self, name: &str, value: Value) -> LuaResult<()> {
        self.lua.globals().set(name, value)
    }

    /// Get a global value
    pub fn get_global(&mut self, name: &str) -> LuaResult<Value<'_>> {
        self.lua.globals().get(name)
    }

    /// Register an event handler
    pub fn on_event(&mut self, event: &str, handler: Function) -> LuaResult<()> {
        let handlers: Table = self.lua.globals().get("_event_handlers").unwrap_or_else(|_| self.lua.create_table().unwrap());
        let event_handlers: Table = handlers.get(event).unwrap_or_else(|_| self.lua.create_table().unwrap());
        let len: usize = event_handlers.raw_len();
        event_handlers.set(len + 1, handler)?;
        handlers.set(event, event_handlers)?;
        self.lua.globals().set("_event_handlers", handlers)?;
        Ok(())
    }

    /// Emit an event
    pub fn emit_event(&mut self, event: &str, args: Vec<Value>) -> LuaResult<()> {
        if let Ok(handlers) = self.lua.globals().get::<&str, Table>("_event_handlers") {
            if let Ok(event_handlers) = handlers.get::<&str, Table>(event) {
                for pair in event_handlers.pairs::<usize, Function>() {
                    let (_, handler) = pair?;
                    handler.call::<Vec<Value>, ()>(args.clone())?;
                }
            }
        }
        Ok(())
    }

    /// Get the Lua state for direct access
    pub fn lua(&mut self) -> &mut Lua {
        &mut self.lua
    }
}

/// Context for script execution with sprite access.
///
/// Thin wrapper over [`context::ScriptContext`], which owns the Lua state and
/// exposes the event and extension systems.
pub struct ScriptContext {
    inner: context::ScriptContext,
}

impl ScriptContext {
    pub fn new() -> CoreResult<Self> {
        Ok(Self {
            inner: context::ScriptContext::new()?,
        })
    }

    pub fn with_sprite(sprite: Sprite) -> CoreResult<Self> {
        Ok(Self {
            inner: context::ScriptContext::with_sprite(sprite)?,
        })
    }

    /// The sprite currently bound to the context.
    pub fn sprite(&self) -> Option<&Arc<Mutex<Sprite>>> {
        self.inner.sprite.as_ref()
    }

    /// Run a script and return its result.
    pub fn run_script(&self, script: &str) -> LuaResult<Value<'_>> {
        self.inner.run_script(script)
    }

    /// Run a script file and return its result.
    pub fn run_file(&self, path: &std::path::Path) -> LuaResult<Value<'_>> {
        self.inner.run_file(path)
    }

    /// Execute a script and discard the result.
    pub fn exec_script(&self, script: &str) -> LuaResult<()> {
        self.inner.run_script(script).map(|_| ())
    }

    /// Execute a script file and discard the result.
    pub fn exec_file(&self, path: &std::path::Path) -> LuaResult<()> {
        self.inner.run_file(path).map(|_| ())
    }

    /// Access the underlying context.
    pub fn context(&self) -> &context::ScriptContext {
        &self.inner
    }
}

/// Sprite API for scripts
pub struct SpriteApi {
    sprite: Arc<Mutex<Sprite>>,
}

impl SpriteApi {
    pub fn new(sprite: Arc<Mutex<Sprite>>) -> Self {
        Self { sprite }
    }

    pub fn width(&self) -> u32 {
        self.sprite.lock().unwrap().metadata.dimensions.w
    }

    pub fn height(&self) -> u32 {
        self.sprite.lock().unwrap().metadata.dimensions.h
    }

    pub fn color_mode(&self) -> ColorMode {
        self.sprite.lock().unwrap().metadata.color_mode
    }

    pub fn frames(&self) -> usize {
        self.sprite.lock().unwrap().frames.order.len()
    }

    pub fn layers(&self) -> Vec<LayerApi> {
        let sprite = self.sprite.lock().unwrap();
        sprite.layers.order.iter()
            .filter_map(|uuid| sprite.layer_data.get(uuid).map(LayerApi::new))
            .collect()
    }

    pub fn save_as(&self, path: &str) -> CoreResult<()> {
        oversprite_parser::write_sprite(&self.sprite.lock().unwrap(), std::path::Path::new(path))
    }
}

/// Layer API for scripts
pub struct LayerApi {
    layer: oversprite_core::Layer,
}

impl LayerApi {
    pub fn new(layer: &oversprite_core::Layer) -> Self {
        Self { layer: layer.clone() }
    }

    pub fn name(&self) -> &str {
        &self.layer.name
    }

    pub fn is_visible(&self) -> bool {
        self.layer.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.layer.visible = visible;
    }

    pub fn opacity(&self) -> u8 {
        self.layer.opacity
    }

    pub fn set_opacity(&mut self, opacity: u8) {
        self.layer.opacity = opacity;
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.layer.blend_mode
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.layer.blend_mode = mode;
    }

    pub fn layer_type(&self) -> LayerType {
        self.layer.layer_type
    }
}

use oversprite_core::{BlendMode, LayerType};

/// Image API for scripts
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

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Rgba) {
        if x >= self.width || y >= self.height { return; }
        let bytes_per_pixel = match self.color_mode {
            ColorMode::Rgba => 4,
            ColorMode::Grayscale => 1,
            ColorMode::Indexed => 1,
            ColorMode::GrayscaleAlpha => 2,
        };
        let idx = ((y * self.width + x) * bytes_per_pixel) as usize;
        match self.color_mode {
            ColorMode::Rgba => {
                self.pixels[idx] = color.r;
                self.pixels[idx + 1] = color.g;
                self.pixels[idx + 2] = color.b;
                self.pixels[idx + 3] = color.a;
            }
            ColorMode::Grayscale => {
                let gray = ((color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000) as u8;
                self.pixels[idx] = gray;
            }
            ColorMode::Indexed => {
                self.pixels[idx] = color.r; // palette index
            }
            ColorMode::GrayscaleAlpha => {
                let gray = ((color.r as u32 * 299 + color.g as u32 * 587 + color.b as u32 * 114) / 1000) as u8;
                self.pixels[idx] = gray;
                self.pixels[idx + 1] = color.a;
            }
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Rgba> {
        if x >= self.width || y >= self.height { return None; }
        let bytes_per_pixel = match self.color_mode {
            ColorMode::Rgba => 4,
            ColorMode::Grayscale => 1,
            ColorMode::Indexed => 1,
            ColorMode::GrayscaleAlpha => 2,
        };
        let idx = ((y * self.width + x) * bytes_per_pixel) as usize;
        match self.color_mode {
            ColorMode::Rgba => Some(Rgba::new(
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            )),
            ColorMode::Grayscale => {
                let gray = self.pixels[idx];
                Some(Rgba::new(gray, gray, gray, 255))
            }
            ColorMode::Indexed => Some(Rgba::new(self.pixels[idx], 0, 0, 255)),
            ColorMode::GrayscaleAlpha => Some(Rgba::new(
                self.pixels[idx],
                self.pixels[idx],
                self.pixels[idx],
                self.pixels[idx + 1],
            )),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.pixels
    }
}

/// Selection API for scripts
pub struct SelectionApi {
    pub bounds: Option<Rect>,
    pub is_empty: bool,
}

impl SelectionApi {
    pub fn new() -> Self {
        Self { bounds: None, is_empty: true }
    }

    pub fn contains(&self, point: Point) -> bool {
        if let Some(bounds) = self.bounds {
            bounds.contains(point.x, point.y)
        } else {
            false
        }
    }
}

use oversprite_core::Rect;