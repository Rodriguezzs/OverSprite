//! OverSprite Scripting - Context Module
//!
//! Script execution context with access to sprite, app, and other globals.

use mlua::{Lua, Result as LuaResult, Value, Function};
use oversprite_core::{CoreError, CoreResult, Sprite, Size, ColorMode, Uuid};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use crate::api::{register_apis, SpriteApi, LayerApi, CelApi, AppApi};
use crate::events::{EventRegistry, EventEmitter, EventType, EventData};
use crate::extensions::ExtensionManager;

/// Convert a Lua error into a [`CoreError`].
fn lua_err(err: mlua::Error) -> CoreError {
    CoreError::Internal(err.to_string())
}

/// Main script context
///
/// Owns the Lua state. mlua 0.9 does not implement `Clone` for `Lua`, so the
/// event registry and the extension manager receive the state as an argument
/// instead of holding a copy of it.
pub struct ScriptContext {
    pub lua: Lua,
    pub sprite: Option<Arc<Mutex<Sprite>>>,
    pub app: AppApi,
    pub event_registry: Arc<Mutex<EventRegistry>>,
    pub event_emitter: EventEmitter,
    pub extension_manager: Option<ExtensionManager>,
}

impl ScriptContext {
    pub fn new() -> CoreResult<Self> {
        let lua = Lua::new();
        register_apis(&lua).map_err(lua_err)?;

        let event_registry = Arc::new(Mutex::new(EventRegistry::new()));
        let event_emitter = EventEmitter::new(event_registry.clone());

        Ok(Self {
            lua,
            sprite: None,
            app: AppApi::new(),
            event_registry,
            event_emitter,
            extension_manager: None,
        })
    }

    pub fn with_sprite(sprite: Sprite) -> CoreResult<Self> {
        let mut ctx = Self::new()?;
        ctx.set_sprite(sprite);
        Ok(ctx)
    }

    pub fn with_sprite_arc(sprite: Arc<Mutex<Sprite>>) -> CoreResult<Self> {
        let mut ctx = Self::new()?;
        ctx.sprite = Some(sprite.clone());
        ctx.app.active_sprite = Some(sprite.clone());
        // Update Lua globals
        ctx.lua.globals().set("app", AppApi::with_sprite(sprite)).map_err(lua_err)?;
        Ok(ctx)
    }

    pub fn set_sprite(&mut self, sprite: Sprite) {
        let sprite_arc = Arc::new(Mutex::new(sprite));
        self.sprite = Some(sprite_arc.clone());
        self.app.active_sprite = Some(sprite_arc.clone());

        // Update Lua globals
        let _ = self.lua.globals().set("app", AppApi::with_sprite(sprite_arc));
    }

    /// Initialize extension manager
    pub fn init_extensions(&mut self, extensions_dir: &Path) -> CoreResult<()> {
        let mut manager = ExtensionManager::new(extensions_dir.to_path_buf())?;
        manager.load_all(&self.lua)?;
        crate::extensions::builtin::load_all(&mut manager, &self.lua)?;
        self.extension_manager = Some(manager);
        Ok(())
    }

    /// Run a script string
    pub fn run_script(&self, script: &str) -> LuaResult<Value<'_>> {
        self.lua.load(script).eval()
    }

    /// Run a script file
    pub fn run_file(&self, path: &Path) -> LuaResult<Value<'_>> {
        let script = std::fs::read_to_string(path)?;
        self.run_script(&script)
    }

    /// Call a function by name
    pub fn call_function<'lua>(&'lua self, name: &str, args: impl mlua::IntoLuaMulti<'lua>) -> LuaResult<Value<'lua>> {
        let func: Function<'lua> = self.lua.globals().get(name)?;
        func.call(args)
    }

    /// Set a global value
    pub fn set_global(&mut self, name: &str, value: Value) -> LuaResult<()> {
        self.lua.globals().set(name, value)
    }

    /// Get a global value
    pub fn get_global(&self, name: &str) -> LuaResult<Value<'_>> {
        self.lua.globals().get(name)
    }

    /// Register event handler
    pub fn on_event(&self, event: EventType, handler: Function) -> LuaResult<()> {
        self.event_registry.lock().unwrap().on(&self.lua, event, handler)
    }

    /// Register one-time event handler
    pub fn once_event(&self, event: EventType, handler: Function) -> LuaResult<()> {
        self.event_registry.lock().unwrap().once(&self.lua, event, handler)
    }

    /// Emit event
    pub fn emit_event(&self, event: EventType, data: EventData) -> LuaResult<()> {
        self.event_registry.lock().unwrap().emit(&self.lua, event, data)
    }

    /// Get sprite API
    pub fn sprite_api(&self) -> Option<SpriteApi> {
        self.sprite.as_ref().map(|s| SpriteApi { sprite: s.clone() })
    }

    /// Get app API
    pub fn app_api(&self) -> &AppApi {
        &self.app
    }

    /// Get mutable app API
    pub fn app_api_mut(&mut self) -> &mut AppApi {
        &mut self.app
    }

    /// Create a new sprite from script
    pub fn create_sprite(&self, width: u32, height: u32, color_mode: ColorMode) -> CoreResult<Sprite> {
        Ok(Sprite::new_with_size(width, height, color_mode))
    }

    /// Load sprite from file
    pub fn load_sprite(&self, path: &Path) -> CoreResult<Sprite> {
        oversprite_parser::read_sprite(path)
    }

    /// Save sprite to file
    pub fn save_sprite(&self, sprite: &Sprite, path: &Path) -> CoreResult<()> {
        oversprite_parser::write_sprite(sprite, path)
    }

    /// Export sprite to image
    pub fn export_sprite(&self, sprite: &Sprite, path: &Path, frame: usize, scale: f32) -> CoreResult<()> {
        use oversprite_renderer::Renderer;
        use std::collections::HashMap;

        let mut renderer = Renderer::new(
            Size::new(
                sprite.metadata.dimensions.w,
                sprite.metadata.dimensions.h,
            ),
            scale,
        )?;

        // Render using the correct API
        let layers: HashMap<_, _> = sprite.layer_data.iter().map(|(k, v)| (*k, v.clone())).collect();
        let cels: HashMap<_, _> = sprite.cel_data.iter().map(|(k, v)| (*k, v.clone())).collect();
        let tilesets: HashMap<_, _> = sprite.tilesets.iter().map(|(k, v)| (*k, v.clone())).collect();

        renderer.render_sprite(
            sprite,
            frame,
            &layers,
            &cels,
            &tilesets,
            &sprite.global_palette,
            None,
        )?;

        // Save as PNG using the renderer's built-in method
        renderer.save_png(path)?;
        Ok(())
    }

    /// Get extension manager
    pub fn extensions(&self) -> Option<&ExtensionManager> {
        self.extension_manager.as_ref()
    }

    /// Get mutable extension manager
    pub fn extensions_mut(&mut self) -> Option<&mut ExtensionManager> {
        self.extension_manager.as_mut()
    }

    /// Run garbage collection
    pub fn gc(&self) -> LuaResult<()> {
        self.lua.gc_collect()
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> LuaResult<usize> {
        Ok(self.lua.used_memory())
    }
}

/// Context for running scripts with specific sprite
pub struct ScriptExecutionContext<'a> {
    pub context: &'a ScriptContext,
    pub sprite: Arc<Mutex<Sprite>>,
    pub frame_index: usize,
    pub layer_uuid: Option<Uuid>,
    pub cel_uuid: Option<Uuid>,
}

impl<'a> ScriptExecutionContext<'a> {
    pub fn new(context: &'a ScriptContext, sprite: Arc<Mutex<Sprite>>) -> Self {
        Self {
            context,
            sprite,
            frame_index: 0,
            layer_uuid: None,
            cel_uuid: None,
        }
    }

    pub fn with_frame(mut self, frame_index: usize) -> Self {
        self.frame_index = frame_index;
        self
    }

    pub fn with_layer(mut self, layer_uuid: Uuid) -> Self {
        self.layer_uuid = Some(layer_uuid);
        self
    }

    pub fn with_cel(mut self, cel_uuid: Uuid) -> Self {
        self.cel_uuid = Some(cel_uuid);
        self
    }

    /// Run script in this context
    pub fn run(&self, script: &str) -> LuaResult<Value<'_>> {
        // Set up context-specific globals
        let globals = self.context.lua.globals();

        // Set active sprite
        let sprite_api = SpriteApi { sprite: self.sprite.clone() };
        globals.set("sprite", sprite_api)?;

        // Set active frame
        globals.set("frame", self.frame_index)?;

        // Set active layer if specified
        if let Some(layer_uuid) = self.layer_uuid {
            let sprite = self.sprite.lock().unwrap();
            if let Some(layer) = sprite.layer_data.get(&layer_uuid) {
                globals.set("layer", LayerApi::new(layer.clone()))?;
            }
        }

        // Set active cel if specified
        if let Some(cel_uuid) = self.cel_uuid {
            let sprite = self.sprite.lock().unwrap();
            if let Some(cel) = sprite.cel_data.get(&cel_uuid) {
                globals.set("cel", CelApi::new(cel.clone()))?;
            }
        }

        // Run script
        self.context.run_script(script)
    }

    /// Run file in this context
    pub fn run_file(&self, path: &Path) -> LuaResult<Value<'_>> {
        let script = std::fs::read_to_string(path)?;
        self.run(&script)
    }
}

/// Builder for creating script contexts
pub struct ScriptContextBuilder {
    sprite: Option<Sprite>,
    extensions_dir: Option<PathBuf>,
    init_script: Option<String>,
}

impl ScriptContextBuilder {
    pub fn new() -> Self {
        Self {
            sprite: None,
            extensions_dir: None,
            init_script: None,
        }
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.sprite = Some(sprite);
        self
    }

    pub fn with_extensions_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.extensions_dir = Some(path.into());
        self
    }

    pub fn with_init_script(mut self, script: String) -> Self {
        self.init_script = Some(script);
        self
    }

    pub fn build(self) -> CoreResult<ScriptContext> {
        let mut ctx = if let Some(sprite) = self.sprite {
            ScriptContext::with_sprite(sprite)?
        } else {
            ScriptContext::new()?
        };

        if let Some(dir) = self.extensions_dir {
            ctx.init_extensions(&dir)?;
        }

        if let Some(script) = self.init_script {
            ctx.run_script(&script).map_err(lua_err)?;
        }

        Ok(ctx)
    }
}

impl Default for ScriptContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export commonly used types
pub use oversprite_core::{Layer, Frame, Cel, Palette, Tag, Slice, Tileset, TileData, Grid, ColorProfile, UserData as CoreUserData, PropertyValue};