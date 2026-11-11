//! Layer definitions for OverSprite

use crate::{BlendMode, ColorMode, CoreError, CoreResult, LayerFlags, Uuid, UserData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Layer type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerType {
    Image,
    Group,
    Tilemap,
}

impl Default for LayerType {
    fn default() -> Self {
        Self::Image
    }
}

impl LayerType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::Image),
            1 => Some(Self::Group),
            2 => Some(Self::Tilemap),
            _ => None,
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            Self::Image => 0,
            Self::Group => 1,
            Self::Tilemap => 2,
        }
    }
}

/// Layer hierarchy depth index (for nested groups)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerDepth(pub u32);

impl Default for LayerDepth {
    fn default() -> Self {
        Self(0)
    }
}

/// Complete layer definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub uuid: Uuid,
    pub name: String,
    pub layer_type: LayerType,
    pub parent: Option<Uuid>,
    pub child_level: u16,
    pub visible: bool,
    pub editable: bool,
    pub lock_movement: bool,
    pub is_background: bool,
    pub prefer_linked_cels: bool,
    pub collapsed: bool,
    pub is_reference: bool,
    pub reference_path: Option<String>,
    pub blend_mode: BlendMode,
    pub opacity: u8,
    pub z_index: i16,
    pub tileset_uuid: Option<Uuid>,
    pub default_width: u32,
    pub default_height: u32,
    pub user_data: UserData,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "Layer".to_string(),
            layer_type: LayerType::Image,
            parent: None,
            child_level: 0,
            visible: true,
            editable: true,
            lock_movement: false,
            is_background: false,
            prefer_linked_cels: false,
            collapsed: false,
            is_reference: false,
            reference_path: None,
            blend_mode: BlendMode::Normal,
            opacity: 255,
            z_index: 0,
            tileset_uuid: None,
            default_width: 0,
            default_height: 0,
            user_data: UserData::default(),
        }
    }
}

impl Layer {
    pub fn new(name: impl Into<String>, layer_type: LayerType) -> Self {
        Self {
            name: name.into(),
            layer_type,
            ..Default::default()
        }
    }

    pub fn new_image(name: impl Into<String>) -> Self {
        Self::new(name, LayerType::Image)
    }

    pub fn new_group(name: impl Into<String>) -> Self {
        Self::new(name, LayerType::Group)
    }

    pub fn new_tilemap(name: impl Into<String>, tileset_uuid: Uuid) -> Self {
        Self {
            name: name.into(),
            layer_type: LayerType::Tilemap,
            tileset_uuid: Some(tileset_uuid),
            ..Default::default()
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self.layer_type, LayerType::Group)
    }

    pub fn is_tilemap(&self) -> bool {
        matches!(self.layer_type, LayerType::Tilemap)
    }

    pub fn is_image(&self) -> bool {
        matches!(self.layer_type, LayerType::Image)
    }

    pub fn effective_opacity(&self, parent_opacity: u8) -> u8 {
        ((self.opacity as u16 * parent_opacity as u16) / 255) as u8
    }

    pub fn flags(&self) -> LayerFlags {
        let mut flags = LayerFlags::empty();
        if self.visible { flags |= LayerFlags::VISIBLE; }
        if self.editable { flags |= LayerFlags::EDITABLE; }
        if self.lock_movement { flags |= LayerFlags::LOCK_MOVEMENT; }
        if self.is_background { flags |= LayerFlags::BACKGROUND; }
        if self.prefer_linked_cels { flags |= LayerFlags::PREFER_LINKED_CELS; }
        if self.collapsed { flags |= LayerFlags::COLLAPSED; }
        if self.is_reference { flags |= LayerFlags::IS_REFERENCE; }
        flags
    }

    pub fn set_flags(&mut self, flags: LayerFlags) {
        self.visible = flags.contains(LayerFlags::VISIBLE);
        self.editable = flags.contains(LayerFlags::EDITABLE);
        self.lock_movement = flags.contains(LayerFlags::LOCK_MOVEMENT);
        self.is_background = flags.contains(LayerFlags::BACKGROUND);
        self.prefer_linked_cels = flags.contains(LayerFlags::PREFER_LINKED_CELS);
        self.collapsed = flags.contains(LayerFlags::COLLAPSED);
        self.is_reference = flags.contains(LayerFlags::IS_REFERENCE);
    }

    pub fn validate(&self, _color_mode: ColorMode) -> CoreResult<()> {
        if self.name.is_empty() {
            return Err(CoreError::ValidationError("Layer name cannot be empty".to_string()));
        }
        if self.is_background && self.layer_type != LayerType::Image {
            return Err(CoreError::ValidationError("Background layer must be image type".to_string()));
        }
        if self.is_tilemap() && self.tileset_uuid.is_none() {
            return Err(CoreError::ValidationError("Tilemap layer requires tileset".to_string()));
        }
        Ok(())
    }
}

/// Layer index for managing layer order and hierarchy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerIndex {
    pub order: Vec<Uuid>,
    pub groups: HashMap<Uuid, Vec<Uuid>>,
}

impl LayerIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_layer(&mut self, uuid: Uuid, parent: Option<Uuid>) {
        self.order.push(uuid);
        if let Some(parent) = parent {
            self.groups.entry(parent).or_default().push(uuid);
        }
    }

    pub fn remove_layer(&mut self, uuid: &Uuid) {
        self.order.retain(|u| u != uuid);
        self.groups.retain(|_, children| {
            children.retain(|u| u != uuid);
            !children.is_empty()
        });
    }

    pub fn move_layer(&mut self, uuid: &Uuid, new_index: usize) {
        if let Some(pos) = self.order.iter().position(|u| u == uuid) {
            let layer = self.order.remove(pos);
            self.order.insert(new_index.min(self.order.len()), layer);
        }
    }

    pub fn children_of(&self, parent: &Uuid) -> &[Uuid] {
        self.groups.get(parent).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn root_layers(&self) -> Vec<&Uuid> {
        self.order.iter()
            .filter(|u| !self.groups.values().any(|children| children.contains(*u)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new_image("Background");
        assert_eq!(layer.name, "Background");
        assert!(layer.is_image());
        assert!(!layer.is_group());
    }

    #[test]
    fn test_layer_flags() {
        let mut layer = Layer::default();
        layer.visible = true;
        layer.editable = false;
        layer.lock_movement = true;

        let flags = layer.flags();
        assert!(flags.contains(LayerFlags::VISIBLE));
        assert!(!flags.contains(LayerFlags::EDITABLE));
        assert!(flags.contains(LayerFlags::LOCK_MOVEMENT));
    }

    #[test]
    fn test_layer_index() {
        let mut index = LayerIndex::new();
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let uuid3 = Uuid::new_v4();

        index.add_layer(uuid1, None);
        index.add_layer(uuid2, Some(uuid1));
        index.add_layer(uuid3, Some(uuid1));

        assert_eq!(index.order.len(), 3);
        assert_eq!(index.children_of(&uuid1).len(), 2);
        assert_eq!(index.root_layers().len(), 1);
    }
}