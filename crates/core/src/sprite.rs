//! Main Sprite structure for OverSprite

use crate::{
    Cel, ColorMode, CoreError, CoreResult, FileFlags,
    Frame, FrameIndex, Layer, LayerIndex, Metadata, Palette,
    Rect, Size, Slice, Tag, Tileset, Uuid, UserData,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sprite header with basic info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteHeader {
    pub uuid: Uuid,
    pub name: String,
    pub version: u32,
    pub minor_version: u32,
    pub created_by: String,
    pub created_at: String,
    pub modified_at: String,
    pub aseprite_compatible: bool,
    pub features: Vec<String>,
    pub compression: CompressionConfig,
    pub flags: FileFlags,
}

impl Default for SpriteHeader {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "Untitled".to_string(),
            version: 1,
            minor_version: 0,
            created_by: "OverSprite".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            aseprite_compatible: true,
            features: vec!["tilesets".into(), "animation_tags".into(), "slices".into(), "user_data".into()],
            compression: CompressionConfig::default(),
            flags: FileFlags::LAYER_OPACITY_VALID | FileFlags::LAYER_BLEND_MODE_VALID | FileFlags::LAYERS_HAVE_UUIDS,
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub metadata: String,   // "deflate", "lz4", "zstd", "none"
    pub pixels: String,
    pub tilemaps: String,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            metadata: "deflate".to_string(),
            pixels: "lz4".to_string(),
            tilemaps: "lz4".to_string(),
        }
    }
}

/// Complete Sprite document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub header: SpriteHeader,
    pub metadata: Metadata,
    pub layers: LayerIndex,
    pub layer_data: HashMap<Uuid, Layer>,
    pub frames: FrameIndex,
    pub frame_data: HashMap<Uuid, Frame>,
    pub cel_data: HashMap<Uuid, Cel>,
    pub global_palette: Palette,
    pub frame_palettes: HashMap<Uuid, Palette>,
    pub tags: Vec<Tag>,
    pub slices: Vec<Slice>,
    pub tilesets: HashMap<Uuid, Tileset>,
    pub user_data: UserData,
    pub thumbnails: HashMap<String, Vec<u8>>,  // size -> PNG bytes
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

impl Sprite {
    pub fn new() -> Self {
        let mut sprite = Self {
            header: SpriteHeader::default(),
            metadata: Metadata::default(),
            layers: LayerIndex::new(),
            layer_data: HashMap::new(),
            frames: FrameIndex::new(),
            frame_data: HashMap::new(),
            cel_data: HashMap::new(),
            global_palette: Palette::default(),
            frame_palettes: HashMap::new(),
            tags: Vec::new(),
            slices: Vec::new(),
            tilesets: HashMap::new(),
            user_data: UserData::default(),
            thumbnails: HashMap::new(),
        };
        sprite.add_default_layer();
        sprite.add_default_frame();
        sprite
    }

    pub fn new_with_size(width: u32, height: u32, color_mode: ColorMode) -> Self {
        let mut sprite = Self::new();
        sprite.metadata.dimensions = Size::new(width, height);
        sprite.metadata.color_mode = color_mode;
        if color_mode == ColorMode::Indexed {
            sprite.global_palette.resize(256);
        }
        sprite
    }

    fn add_default_layer(&mut self) {
        let layer = Layer::new_image("Layer 1");
        let uuid = layer.uuid;
        self.layer_data.insert(uuid, layer);
        self.layers.add_layer(uuid, None);
    }

    fn add_default_frame(&mut self) {
        let frame = Frame::new(self.metadata.default_frame_duration_ms);
        let uuid = frame.uuid;
        self.frame_data.insert(uuid, frame);
        self.frames.add_frame(uuid);
    }

    // Layer operations
    pub fn add_layer(&mut self, layer: Layer, parent: Option<Uuid>) -> Uuid {
        let uuid = layer.uuid;
        layer.validate(self.metadata.color_mode).unwrap();
        self.layer_data.insert(uuid, layer);
        self.layers.add_layer(uuid, parent);
        uuid
    }

    pub fn remove_layer(&mut self, uuid: &Uuid) -> CoreResult<()> {
        if let Some(layer) = self.layer_data.get(uuid) {
            if layer.is_background {
                return Err(CoreError::ValidationError("Cannot remove background layer".into()));
            }
        }
        self.layer_data.remove(uuid);
        self.layers.remove_layer(uuid);
        // Remove associated cels
        for frame in self.frame_data.values_mut() {
            frame.remove_cel(uuid);
        }
        Ok(())
    }

    pub fn get_layer(&self, uuid: &Uuid) -> Option<&Layer> {
        self.layer_data.get(uuid)
    }

    pub fn get_layer_mut(&mut self, uuid: &Uuid) -> Option<&mut Layer> {
        self.layer_data.get_mut(uuid)
    }

    pub fn layer_order(&self) -> &[Uuid] {
        &self.layers.order
    }

    // Frame operations
    pub fn add_frame(&mut self, duration_ms: u32) -> Uuid {
        let frame = Frame::new(duration_ms);
        let uuid = frame.uuid;
        self.frame_data.insert(uuid, frame);
        self.frames.add_frame(uuid);
        uuid
    }

    pub fn remove_frame(&mut self, uuid: &Uuid) -> CoreResult<()> {
        if self.frames.len() <= 1 {
            return Err(CoreError::ValidationError("Cannot remove last frame".into()));
        }
        self.frame_data.remove(uuid);
        self.frames.remove_frame(uuid);
        Ok(())
    }

    pub fn get_frame(&self, uuid: &Uuid) -> Option<&Frame> {
        self.frame_data.get(uuid)
    }

    pub fn get_frame_mut(&mut self, uuid: &Uuid) -> Option<&mut Frame> {
        self.frame_data.get_mut(uuid)
    }

    pub fn frame_order(&self) -> &[Uuid] {
        &self.frames.order
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    // Cel operations
    pub fn add_cel(&mut self, cel: Cel) -> Uuid {
        let uuid = cel.uuid;
        let layer_uuid = cel.layer_uuid;
        self.cel_data.insert(uuid, cel);
        if let Some(frame) = self.frame_data.get_mut(&layer_uuid) {
            frame.add_cel(uuid);
        }
        uuid
    }

    pub fn get_cel(&self, uuid: &Uuid) -> Option<&Cel> {
        self.cel_data.get(uuid)
    }

    pub fn get_cel_mut(&mut self, uuid: &Uuid) -> Option<&mut Cel> {
        self.cel_data.get_mut(uuid)
    }

    pub fn cels_in_frame(&self, frame_uuid: &Uuid) -> Vec<&Cel> {
        self.frame_data.get(frame_uuid)
            .map(|f| f.cels.iter().filter_map(|c| self.cel_data.get(c)).collect())
            .unwrap_or_default()
    }

    // Palette operations
    pub fn get_palette(&self, frame_uuid: Option<&Uuid>) -> &Palette {
        if let Some(frame_uuid) = frame_uuid {
            self.frame_palettes.get(frame_uuid).unwrap_or(&self.global_palette)
        } else {
            &self.global_palette
        }
    }

    pub fn get_palette_mut(&mut self, frame_uuid: Option<&Uuid>) -> &mut Palette {
        if let Some(frame_uuid) = frame_uuid {
            self.frame_palettes.entry(*frame_uuid).or_insert_with(|| self.global_palette.clone())
        } else {
            &mut self.global_palette
        }
    }

    // Validation
    pub fn validate(&self) -> CoreResult<()> {
        self.metadata.validate()?;
        self.global_palette.validate()?;

        // Check background layer
        let bg_count = self.layer_data.values().filter(|l| l.is_background).count();
        if bg_count > 1 {
            return Err(CoreError::ValidationError("Only one background layer allowed".into()));
        }

        // Validate all layers
        for layer in self.layer_data.values() {
            layer.validate(self.metadata.color_mode)?;
        }

        // Validate all frames
        for frame in self.frame_data.values() {
            for cel_uuid in &frame.cels {
                if let Some(cel) = self.cel_data.get(cel_uuid) {
                    cel.validate()?;
                }
            }
        }

        Ok(())
    }

    // Bounds
    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.metadata.dimensions.w, self.metadata.dimensions.h)
    }

    pub fn dimensions(&self) -> Size {
        self.metadata.dimensions
    }

    pub fn color_mode(&self) -> ColorMode {
        self.metadata.color_mode
    }
}

// Need to import chrono for timestamps
mod chrono_import {
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite::new();
        assert_eq!(sprite.frame_count(), 1);
        assert_eq!(sprite.layers.order.len(), 1);
    }

    #[test]
    fn test_sprite_with_size() {
        let sprite = Sprite::new_with_size(320, 180, ColorMode::Rgba);
        assert_eq!(sprite.dimensions().w, 320);
        assert_eq!(sprite.dimensions().h, 180);
        assert_eq!(sprite.color_mode(), ColorMode::Rgba);
    }

    #[test]
    fn test_layer_management() {
        let mut sprite = Sprite::new();
        let layer = Layer::new_image("New Layer");
        let uuid = layer.uuid;
        sprite.add_layer(layer, None);
        assert!(sprite.get_layer(&uuid).is_some());
    }
}