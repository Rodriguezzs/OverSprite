//! OverSprite Scripting - Extensions Module
//!
//! Extension system for loading and managing Lua-based extensions/packages.

use mlua::{Lua, Result as LuaResult, Table, Function};
use oversprite_core::{CoreError, CoreResult};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

/// Extension
pub struct Extension {
    pub manifest: ExtensionManifest,
    pub path: PathBuf,
    pub exports_key: String, // Key in Lua registry
    pub enabled: bool,
}

impl Extension {
    /// Get the exports table from the Lua registry
    pub fn get_exports<'lua>(&self, lua: &'lua Lua) -> LuaResult<Table<'lua>> {
        lua.named_registry_value(&self.exports_key)
    }
}

/// Extension manifest (package.json equivalent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub main: String,
    pub api_version: String,
    pub dependencies: HashMap<String, String>,
    pub permissions: Vec<Permission>,
    pub contributes: ExtensionContributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionContributes {
    pub commands: Vec<CommandContribution>,
    pub menus: Vec<MenuContribution>,
    pub keybindings: Vec<KeybindingContribution>,
    pub palettes: Vec<PaletteContribution>,
    pub tools: Vec<ToolContribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContribution {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuContribution {
    pub menu: String,
    pub command: String,
    pub when: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingContribution {
    pub command: String,
    pub key: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteContribution {
    pub id: String,
    pub name: String,
    pub colors: Vec<[u8; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContribution {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub cursor: Option<String>,
}

/// Extension manager
///
/// `Lua` is not `Clone` in mlua 0.9, so the manager does not own a copy of the
/// state: the [`Lua`] handle is passed to every method that needs it.
pub struct ExtensionManager {
    pub extensions: HashMap<String, Extension>,
    pub extensions_dir: PathBuf,
}

impl ExtensionManager {
    pub fn new(extensions_dir: PathBuf) -> CoreResult<Self> {
        fs::create_dir_all(&extensions_dir)?;
        Ok(Self {
            extensions: HashMap::new(),
            extensions_dir,
        })
    }

    /// Load all extensions from the extensions directory
    pub fn load_all(&mut self, lua: &Lua) -> CoreResult<()> {
        for entry in fs::read_dir(&self.extensions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) = self.load_extension(lua, &path) {
                    eprintln!("Failed to load extension {:?}: {}", path, e);
                }
            }
        }
        Ok(())
    }

    /// Load a single extension from a directory
    pub fn load_extension(&mut self, lua: &Lua, path: &Path) -> CoreResult<()> {
        let manifest_path = path.join("package.json");
        if !manifest_path.exists() {
            return Err(CoreError::Internal("package.json not found".into()));
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        // Check API version compatibility
        if !self.is_api_compatible(&manifest.api_version) {
            return Err(CoreError::Internal(format!(
                "Extension {} requires API version {}, but current is 1.0",
                manifest.name, manifest.api_version
            )));
        }

        // Load main script
        let main_path = path.join(&manifest.main);
        if !main_path.exists() {
            return Err(CoreError::Internal(format!("Main file {} not found", manifest.main)));
        }

        let script = fs::read_to_string(&main_path)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        let exports: Table = lua.load(&script).eval()
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        // Store exports in Lua registry
        let exports_key = format!("extension_exports_{}", manifest.name);
        lua.set_named_registry_value(&exports_key, exports)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let extension = Extension {
            manifest: manifest.clone(),
            path: path.to_path_buf(),
            exports_key,
            enabled: true,
        };

        // Run activation function if present
        let exports_table = extension.get_exports(lua)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        if let Ok(activate) = exports_table.get::<_, Function>("activate") {
            activate.call::<_, ()>(())
                .map_err(|e| CoreError::Internal(e.to_string()))?;
        }

        self.extensions.insert(manifest.name.clone(), extension);
        Ok(())
    }

    /// Unload an extension
    pub fn unload_extension(&mut self, lua: &Lua, name: &str) -> CoreResult<()> {
        if let Some(ext) = self.extensions.remove(name) {
            if let Ok(exports) = ext.get_exports(lua) {
                if let Ok(deactivate) = exports.get::<_, Function>("deactivate") {
                    deactivate.call::<_, ()>(())
                        .map_err(|e| CoreError::Internal(e.to_string()))?;
                }
            }
            // Clean up registry
            lua.unset_named_registry_value(&ext.exports_key)
                .map_err(|e| CoreError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Enable/disable extension
    pub fn set_enabled(&mut self, lua: &Lua, name: &str, enabled: bool) -> CoreResult<()> {
        if let Some(ext) = self.extensions.get_mut(name) {
            ext.enabled = enabled;
            if let Ok(exports) = ext.get_exports(lua) {
                if enabled {
                    if let Ok(activate) = exports.get::<_, Function>("activate") {
                        activate.call::<_, ()>(())
                            .map_err(|e| CoreError::Internal(e.to_string()))?;
                    }
                } else {
                    if let Ok(deactivate) = exports.get::<_, Function>("deactivate") {
                        deactivate.call::<_, ()>(())
                            .map_err(|e| CoreError::Internal(e.to_string()))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get extension by name
    pub fn get_extension(&self, name: &str) -> Option<&Extension> {
        self.extensions.get(name)
    }

    /// List all extensions
    pub fn list_extensions(&self) -> Vec<&Extension> {
        self.extensions.values().collect()
    }

    /// Check API version compatibility
    fn is_api_compatible(&self, required: &str) -> bool {
        // Simple semver check - in reality would be more sophisticated
        required.starts_with("1.")
    }

    /// Get contributed commands
    pub fn get_commands(&self) -> Vec<CommandContribution> {
        self.extensions.values()
            .filter(|e| e.enabled)
            .flat_map(|e| e.manifest.contributes.commands.clone())
            .collect()
    }

    /// Get contributed menus
    pub fn get_menus(&self) -> Vec<MenuContribution> {
        self.extensions.values()
            .filter(|e| e.enabled)
            .flat_map(|e| e.manifest.contributes.menus.clone())
            .collect()
    }

    /// Get contributed keybindings
    pub fn get_keybindings(&self) -> Vec<KeybindingContribution> {
        self.extensions.values()
            .filter(|e| e.enabled)
            .flat_map(|e| e.manifest.contributes.keybindings.clone())
            .collect()
    }

    /// Get contributed palettes
    pub fn get_palettes(&self) -> Vec<PaletteContribution> {
        self.extensions.values()
            .filter(|e| e.enabled)
            .flat_map(|e| e.manifest.contributes.palettes.clone())
            .collect()
    }

    /// Get contributed tools
    pub fn get_tools(&self) -> Vec<ToolContribution> {
        self.extensions.values()
            .filter(|e| e.enabled)
            .flat_map(|e| e.manifest.contributes.tools.clone())
            .collect()
    }

    /// Create extension template
    pub fn create_template(&self, name: &str, path: &Path) -> CoreResult<()> {
        fs::create_dir_all(path)?;

        let manifest = ExtensionManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            author: "Your Name".to_string(),
            description: "A OverSprite extension".to_string(),
            main: "main.lua".to_string(),
            api_version: "1.0".to_string(),
            dependencies: HashMap::new(),
            permissions: vec![],
            contributes: ExtensionContributes::default(),
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(path.join("package.json"), manifest_json)?;

        let main_lua = r#"-- OverSprite Extension Template

local M = {}

function M.activate()
    print("Extension activated: " .. _EXTENSION_NAME)
end

function M.deactivate()
    print("Extension deactivated: " .. _EXTENSION_NAME)
end

-- Register a command
-- app.commands.register("myExtension.hello", function()
--     app.alert("Hello from extension!")
-- end)

return M
"#;

        fs::write(path.join("main.lua"), main_lua)?;

        Ok(())
    }
}

/// Built-in extensions (shipped with OverSprite)
pub mod builtin {
    use super::*;

    /// Load built-in extensions
    pub fn load_all(manager: &mut ExtensionManager, lua: &Lua) -> CoreResult<()> {
        // Palette extensions
        load_palette_extensions(manager)?;

        // Tool extensions
        load_tool_extensions(manager)?;

        // Export format extensions
        load_export_extensions(manager)?;

        let _ = lua;
        Ok(())
    }

    fn load_palette_extensions(_manager: &mut ExtensionManager) -> CoreResult<()> {
        // Would load built-in palette packs
        Ok(())
    }

    fn load_tool_extensions(_manager: &mut ExtensionManager) -> CoreResult<()> {
        // Would load built-in tools
        Ok(())
    }

    fn load_export_extensions(_manager: &mut ExtensionManager) -> CoreResult<()> {
        // Would load export format handlers
        Ok(())
    }
}