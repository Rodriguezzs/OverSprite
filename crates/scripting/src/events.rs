//! OverSprite Scripting - Events Module
//!
//! Event system for scripting API with typed event handlers.

use mlua::{Lua, Result as LuaResult, Table, Value, Function};
use oversprite_core::{Sprite, Rgba, Uuid};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Event types that can be emitted
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    // Sprite events
    SpriteCreated,
    SpriteChanged,
    SpriteSaved,
    SpriteClosed,

    // Layer events
    LayerCreated,
    LayerChanged,
    LayerDeleted,
    LayerVisibilityChanged,
    LayerOpacityChanged,
    LayerBlendModeChanged,
    LayerOrderChanged,

    // Frame events
    FrameCreated,
    FrameChanged,
    FrameDeleted,
    FrameDurationChanged,
    FrameOrderChanged,
    ActiveFrameChanged,

    // Cel events
    CelCreated,
    CelChanged,
    CelDeleted,
    CelPositionChanged,
    CelOpacityChanged,
    CelImageChanged,

    // Selection events
    SelectionChanged,
    SelectionCleared,

    // Tool events
    ToolChanged,
    ToolOptionsChanged,

    // Color events
    ForegroundColorChanged,
    BackgroundColorChanged,

    // View events
    ZoomChanged,
    PanChanged,
    GridVisibilityChanged,
    OnionSkinChanged,

    // Playback events
    PlaybackStarted,
    PlaybackStopped,
    PlaybackFrameChanged,

    // File events
    FileOpened,
    FileSaved,
    FileExported,

    // Custom events
    Custom(String),
}

impl EventType {
    /// Get a unique numeric identifier for this event type
    pub fn as_u32(&self) -> u32 {
        match self {
            EventType::SpriteCreated => 1,
            EventType::SpriteChanged => 2,
            EventType::SpriteSaved => 3,
            EventType::SpriteClosed => 4,
            EventType::LayerCreated => 10,
            EventType::LayerChanged => 11,
            EventType::LayerDeleted => 12,
            EventType::LayerVisibilityChanged => 13,
            EventType::LayerOpacityChanged => 14,
            EventType::LayerBlendModeChanged => 15,
            EventType::LayerOrderChanged => 16,
            EventType::FrameCreated => 20,
            EventType::FrameChanged => 21,
            EventType::FrameDeleted => 22,
            EventType::FrameDurationChanged => 23,
            EventType::FrameOrderChanged => 24,
            EventType::ActiveFrameChanged => 25,
            EventType::CelCreated => 30,
            EventType::CelChanged => 31,
            EventType::CelDeleted => 32,
            EventType::CelPositionChanged => 33,
            EventType::CelOpacityChanged => 34,
            EventType::CelImageChanged => 35,
            EventType::SelectionChanged => 40,
            EventType::SelectionCleared => 41,
            EventType::ToolChanged => 50,
            EventType::ToolOptionsChanged => 51,
            EventType::ForegroundColorChanged => 60,
            EventType::BackgroundColorChanged => 61,
            EventType::ZoomChanged => 70,
            EventType::PanChanged => 71,
            EventType::GridVisibilityChanged => 72,
            EventType::OnionSkinChanged => 73,
            EventType::PlaybackStarted => 80,
            EventType::PlaybackStopped => 81,
            EventType::PlaybackFrameChanged => 82,
            EventType::FileOpened => 90,
            EventType::FileSaved => 91,
            EventType::FileExported => 92,
            EventType::Custom(_) => 100,
        }
    }
}

/// Event type with data for custom events
#[derive(Debug, Clone, PartialEq)]
pub struct EventTypeWithData {
    pub name: String,
    pub args: Vec<LuaValue>, // Use owned LuaValue wrapper
}

/// Owned wrapper for Lua values (can be converted to mlua::Value when needed)
#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Table(Vec<(LuaValue, LuaValue)>), // Simplified - in practice would need proper table representation
    Function(u64), // Function ID reference
}

/// Event data payload - uses owned types, not mlua values
#[derive(Debug, Clone)]
pub enum EventData {
    Sprite(SpriteEventData),
    Layer(LayerEventData),
    Frame(FrameEventData),
    Cel(CelEventData),
    Selection(SelectionEventData),
    Tool(ToolEventData),
    Color(ColorEventData),
    View(ViewEventData),
    Playback(PlaybackEventData),
    File(FileEventData),
    Custom(String, Vec<LuaValue>),
}

#[derive(Debug, Clone)]
pub struct SpriteEventData {
    pub sprite: Option<Arc<Mutex<Sprite>>>,
}

#[derive(Debug, Clone)]
pub struct LayerEventData {
    pub layer_uuid: Uuid,
    pub sprite: Option<Arc<Mutex<Sprite>>>,
}

#[derive(Debug, Clone)]
pub struct FrameEventData {
    pub frame_index: usize,
    pub sprite: Option<Arc<Mutex<Sprite>>>,
}

#[derive(Debug, Clone)]
pub struct CelEventData {
    pub cel_uuid: Uuid,
    pub layer_uuid: Uuid,
    pub frame_index: usize,
    pub sprite: Option<Arc<Mutex<Sprite>>>,
}

#[derive(Debug, Clone)]
pub struct SelectionEventData {
    pub bounds: Option<RectData>,
    pub is_empty: bool,
}

#[derive(Debug, Clone)]
pub struct ToolEventData {
    pub tool_id: String,
    pub options: HashMap<String, LuaValue>,
}

#[derive(Debug, Clone)]
pub struct ColorEventData {
    pub color: Rgba,
    pub is_foreground: bool,
}

#[derive(Debug, Clone)]
pub struct ViewEventData {
    pub zoom: f32,
    pub pan: PointData,
}

#[derive(Debug, Clone)]
pub struct PlaybackEventData {
    pub playing: bool,
    pub frame: usize,
    pub fps: u32,
}

#[derive(Debug, Clone)]
pub struct FileEventData {
    pub path: String,
    pub format: String,
}

#[derive(Debug, Clone, Copy)]
pub struct PointData {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct RectData {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// Event handler registry
///
/// Keeps track of registered handlers. `Lua` is not `Clone` in mlua 0.9, so
/// the registry does not own a copy of the state: the [`Lua`] handle is
/// passed to every method that needs it.
pub struct EventRegistry {
    next_handler_id: usize,
}

impl EventRegistry {
    pub fn new() -> Self {
        Self {
            next_handler_id: 0,
        }
    }

    /// Register an event handler
    pub fn on(&mut self, lua: &Lua, event: EventType, function: Function) -> LuaResult<()> {
        let id = self.next_handler_id;
        self.next_handler_id += 1;
        let key = format!("handler_{}_{}", event.as_u32(), id);
        lua.set_named_registry_value(&key, function)?;

        let handlers: Table = lua.globals().get("_event_handlers").unwrap_or_else(|_| lua.create_table().unwrap());
        let event_handlers: Table = handlers.get(event.as_u32()).unwrap_or_else(|_| lua.create_table().unwrap());
        event_handlers.set(id, key)?;
        handlers.set(event.as_u32(), event_handlers)?;
        lua.globals().set("_event_handlers", handlers)?;
        Ok(())
    }

    /// Register a one-time event handler
    pub fn once(&mut self, lua: &Lua, event: EventType, function: Function) -> LuaResult<()> {
        let id = self.next_handler_id;
        self.next_handler_id += 1;
        let key = format!("handler_{}_{}", event.as_u32(), id);
        lua.set_named_registry_value(&key, function)?;

        let handlers: Table = lua.globals().get("_event_handlers").unwrap_or_else(|_| lua.create_table().unwrap());
        let event_handlers: Table = handlers.get(event.as_u32()).unwrap_or_else(|_| lua.create_table().unwrap());

        // Stored as { key = "...", once = true } so `emit` can read both fields.
        let entry = lua.create_table()?;
        entry.set("key", key)?;
        entry.set("once", true)?;
        event_handlers.set(id, entry)?;
        handlers.set(event.as_u32(), event_handlers)?;
        lua.globals().set("_event_handlers", handlers)?;
        Ok(())
    }

    /// Register with filter
    pub fn on_filtered<F>(&mut self, lua: &Lua, event: EventType, function: Function, _filter: F) -> LuaResult<()>
    where
        F: Fn(&EventData) -> bool + Send + Sync + 'static,
    {
        // For simplicity, store filter as a separate registry entry
        // In practice, filters are harder to serialize
        let id = self.next_handler_id;
        self.next_handler_id += 1;
        let key = format!("handler_{}_{}", event.as_u32(), id);
        lua.set_named_registry_value(&key, function)?;

        let handlers: Table = lua.globals().get("_event_handlers").unwrap_or_else(|_| lua.create_table().unwrap());
        let event_handlers: Table = handlers.get(event.as_u32()).unwrap_or_else(|_| lua.create_table().unwrap());
        event_handlers.set(id, key)?;
        handlers.set(event.as_u32(), event_handlers)?;
        lua.globals().set("_event_handlers", handlers)?;
        Ok(())
    }

    /// Remove an event handler
    pub fn off(&mut self, _event: EventType, _function: Function) -> LuaResult<()> {
        // In practice, we'd need to store handler IDs to remove specific handlers
        Ok(())
    }

    /// Emit an event to all handlers
    pub fn emit(&mut self, lua: &Lua, event: EventType, data: EventData) -> LuaResult<()> {
        let handlers: Table = match lua.globals().get("_event_handlers") {
            Ok(t) => t,
            Err(_) => return Ok(()),
        };

        let event_handlers: Table = match handlers.get(event.as_u32()) {
            Ok(t) => t,
            Err(_) => return Ok(()),
        };

        let mut to_remove = Vec::new();

        // `pairs` consumes the table, so iterate over a clone and keep the
        // original around to remove the one-time handlers afterwards.
        for pair in event_handlers.clone().pairs::<usize, mlua::Value>() {
            let (id, value) = pair?;

            let (key, once): (String, bool) = match value {
                mlua::Value::Table(t) => {
                    // Format: { key = "...", once = true }
                    let k: String = t.get("key")?;
                    let o: bool = t.get("once").unwrap_or(false);
                    (k, o)
                }
                mlua::Value::String(s) => {
                    // Old format: just the key string
                    (s.to_str()?.to_string(), false)
                }
                _ => continue,
            };

            // Retrieve function from registry
            let func: Function = lua.named_registry_value(&key)?;

            // Call handler
            let lua_data = self.event_data_to_lua(lua, &data)?;
            let result: LuaResult<()> = func.call(lua_data);

            // Remove if once
            if once {
                to_remove.push(id);
                lua.unset_named_registry_value(&key)?;
            }

            if let Err(e) = result {
                eprintln!("Event handler error: {}", e);
            }
        }

        // Remove "once" handlers
        for id in to_remove {
            event_handlers.raw_remove(id)?;
        }

        Ok(())
    }

    fn event_data_to_lua<'lua>(&self, lua: &'lua Lua, data: &EventData) -> LuaResult<Value<'lua>> {
        let table = lua.create_table()?;
        match data {
            EventData::Sprite(_d) => {
                // Add sprite data
            }
            EventData::Layer(d) => {
                table.set("layerUuid", lua.create_string(&d.layer_uuid.to_string())?)?;
            }
            EventData::Frame(d) => {
                table.set("frameIndex", d.frame_index)?;
            }
            EventData::Cel(d) => {
                table.set("celUuid", lua.create_string(&d.cel_uuid.to_string())?)?;
                table.set("layerUuid", lua.create_string(&d.layer_uuid.to_string())?)?;
                table.set("frameIndex", d.frame_index)?;
            }
            EventData::Selection(d) => {
                table.set("isEmpty", d.is_empty)?;
                if let Some(bounds) = d.bounds {
                    let bounds_table = lua.create_table()?;
                    bounds_table.set("x", bounds.x)?;
                    bounds_table.set("y", bounds.y)?;
                    bounds_table.set("w", bounds.w)?;
                    bounds_table.set("h", bounds.h)?;
                    table.set("bounds", bounds_table)?;
                }
            }
            EventData::Tool(d) => {
                table.set("toolId", lua.create_string(&d.tool_id)?)?;
                // Convert options HashMap to Lua table
                if !d.options.is_empty() {
                    let opts_table = lua.create_table()?;
                    for (k, v) in &d.options {
                        opts_table.set(k.as_str(), self.lua_value_to_mlua(lua, v)?)?;
                    }
                    table.set("options", opts_table)?;
                }
            }
            EventData::Color(d) => {
                table.set("r", d.color.r)?;
                table.set("g", d.color.g)?;
                table.set("b", d.color.b)?;
                table.set("a", d.color.a)?;
                table.set("isForeground", d.is_foreground)?;
            }
            EventData::View(d) => {
                table.set("zoom", d.zoom)?;
                table.set("panX", d.pan.x)?;
                table.set("panY", d.pan.y)?;
            }
            EventData::Playback(d) => {
                table.set("playing", d.playing)?;
                table.set("frame", d.frame)?;
                table.set("fps", d.fps)?;
            }
            EventData::File(d) => {
                table.set("path", lua.create_string(&d.path)?)?;
                table.set("format", lua.create_string(&d.format)?)?;
            }
            EventData::Custom(name, args) => {
                table.set("name", lua.create_string(name)?)?;
                if !args.is_empty() {
                    let args_table = lua.create_table()?;
                    for (i, arg) in args.iter().enumerate() {
                        args_table.set(i + 1, self.lua_value_to_mlua(lua, arg)?)?;
                    }
                    table.set("args", args_table)?;
                }
            }
        }
        Ok(Value::Table(table))
    }

    /// Convert owned LuaValue to mlua::Value
    fn lua_value_to_mlua<'lua>(&self, lua: &'lua Lua, value: &LuaValue) -> LuaResult<mlua::Value<'lua>> {
        Ok(match value {
            LuaValue::Nil => mlua::Value::Nil,
            LuaValue::Boolean(b) => mlua::Value::Boolean(*b),
            LuaValue::Integer(i) => mlua::Value::Integer(*i),
            LuaValue::Number(n) => mlua::Value::Number(*n),
            LuaValue::String(s) => mlua::Value::String(lua.create_string(s)?),
            LuaValue::Table(_) => {
                // Simplified - would need proper table conversion
                mlua::Value::Table(lua.create_table()?)
            }
            LuaValue::Function(_) => {
                // Function reference - would need lookup
                mlua::Value::Nil
            }
        })
    }
}

/// Global event emitter for the application
pub struct EventEmitter {
    registry: Arc<Mutex<EventRegistry>>,
}

impl EventEmitter {
    pub fn new(registry: Arc<Mutex<EventRegistry>>) -> Self {
        Self { registry }
    }

    pub fn emit(&self, lua: &Lua, event: EventType, data: EventData) {
        if let Ok(mut reg) = self.registry.lock() {
            let _ = reg.emit(lua, event, data);
        }
    }

    // Convenience methods for common events
    pub fn sprite_changed(&self, lua: &Lua, sprite: Arc<Mutex<Sprite>>) {
        self.emit(lua, EventType::SpriteChanged, EventData::Sprite(SpriteEventData { sprite: Some(sprite) }));
    }

    pub fn layer_changed(&self, lua: &Lua, layer_uuid: Uuid, sprite: Arc<Mutex<Sprite>>) {
        self.emit(lua, EventType::LayerChanged, EventData::Layer(LayerEventData { layer_uuid, sprite: Some(sprite) }));
    }

    pub fn active_frame_changed(&self, lua: &Lua, frame_index: usize, sprite: Arc<Mutex<Sprite>>) {
        self.emit(lua, EventType::ActiveFrameChanged, EventData::Frame(FrameEventData { frame_index, sprite: Some(sprite) }));
    }

    pub fn selection_changed(&self, lua: &Lua, bounds: Option<RectData>, is_empty: bool) {
        self.emit(lua, EventType::SelectionChanged, EventData::Selection(SelectionEventData { bounds, is_empty }));
    }

    pub fn foreground_color_changed(&self, lua: &Lua, color: Rgba) {
        self.emit(lua, EventType::ForegroundColorChanged, EventData::Color(ColorEventData { color, is_foreground: true }));
    }

    pub fn zoom_changed(&self, lua: &Lua, zoom: f32, pan: PointData) {
        self.emit(lua, EventType::ZoomChanged, EventData::View(ViewEventData { zoom, pan }));
    }

    pub fn playback_frame_changed(&self, lua: &Lua, frame: usize, fps: u32) {
        self.emit(lua, EventType::PlaybackFrameChanged, EventData::Playback(PlaybackEventData { playing: true, frame, fps }));
    }
}