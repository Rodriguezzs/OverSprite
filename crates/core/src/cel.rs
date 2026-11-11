//! Cel definitions for OverSprite

use crate::{CoreError, CoreResult, Fixed16_16, Point, Rect, Uuid, UserData};
use serde::{Deserialize, Serialize};

/// Cel type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CelType {
    Raw,
    Linked,
    Compressed,
    Tilemap,
}

impl Default for CelType {
    fn default() -> Self {
        Self::Compressed
    }
}

impl CelType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::Raw),
            1 => Some(Self::Linked),
            2 => Some(Self::Compressed),
            3 => Some(Self::Tilemap),
            _ => None,
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            Self::Raw => 0,
            Self::Linked => 1,
            Self::Compressed => 2,
            Self::Tilemap => 3,
        }
    }
}

/// Cel extra data for sub-pixel positioning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CelExtra {
    pub precise_bounds: bool,
    pub precise_x: Fixed16_16,
    pub precise_y: Fixed16_16,
    pub precise_width: Fixed16_16,
    pub precise_height: Fixed16_16,
}

/// Cel (layer content for a specific frame)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cel {
    pub uuid: Uuid,
    pub layer_uuid: Uuid,
    pub position: Point,
    pub opacity: u8,
    pub cel_type: CelType,
    pub z_index: i16,
    pub bounds: Rect,
    pub linked_frame_uuid: Option<Uuid>,
    pub tileset_uuid: Option<Uuid>,
    pub has_extra: bool,
    pub extra: CelExtra,
    pub user_data: UserData,
}

impl Default for Cel {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            layer_uuid: Uuid::nil(),
            position: Point::zero(),
            opacity: 255,
            cel_type: CelType::Compressed,
            z_index: 0,
            bounds: Rect::empty(),
            linked_frame_uuid: None,
            tileset_uuid: None,
            has_extra: false,
            extra: CelExtra::default(),
            user_data: UserData::default(),
        }
    }
}

impl Cel {
    pub fn new(layer_uuid: Uuid, position: Point, bounds: Rect) -> Self {
        Self {
            layer_uuid,
            position,
            bounds,
            ..Default::default()
        }
    }

    pub fn new_linked(layer_uuid: Uuid, linked_frame: Uuid) -> Self {
        Self {
            layer_uuid,
            cel_type: CelType::Linked,
            linked_frame_uuid: Some(linked_frame),
            ..Default::default()
        }
    }

    pub fn new_tilemap(layer_uuid: Uuid, tileset_uuid: Uuid, bounds: Rect) -> Self {
        Self {
            layer_uuid,
            cel_type: CelType::Tilemap,
            tileset_uuid: Some(tileset_uuid),
            bounds,
            ..Default::default()
        }
    }

    pub fn is_linked(&self) -> bool {
        matches!(self.cel_type, CelType::Linked)
    }

    pub fn is_tilemap(&self) -> bool {
        matches!(self.cel_type, CelType::Tilemap)
    }

    pub fn effective_opacity(&self, layer_opacity: u8) -> u8 {
        ((self.opacity as u16 * layer_opacity as u16) / 255) as u8
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.is_linked() && self.linked_frame_uuid.is_none() {
            return Err(CoreError::ValidationError("Linked cel requires linked_frame_uuid".to_string()));
        }
        if self.is_tilemap() && self.tileset_uuid.is_none() {
            return Err(CoreError::ValidationError("Tilemap cel requires tileset_uuid".to_string()));
        }
        if self.bounds.w == 0 || self.bounds.h == 0 {
            return Err(CoreError::ValidationError("Cel bounds cannot be empty".to_string()));
        }
        Ok(())
    }
}

/// Raw pixel data for a cel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelPixelData {
    pub format: String,  // "rgba", "grayscale", "indexed"
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,   // Compressed with LZ4
    pub uncompressed_size: u32,
}

impl CelPixelData {
    pub fn new(format: impl Into<String>, width: u32, height: u32, data: Vec<u8>) -> Self {
        let uncompressed_size = data.len() as u32;
        Self {
            format: format.into(),
            width,
            height,
            data,
            uncompressed_size,
        }
    }

    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn expected_size(&self, bytes_per_pixel: usize) -> usize {
        self.width as usize * self.height as usize * bytes_per_pixel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cel_creation() {
        let layer_uuid = Uuid::new_v4();
        let cel = Cel::new(layer_uuid, Point::new(10, 20), Rect::new(10, 20, 100, 100));
        assert_eq!(cel.layer_uuid, layer_uuid);
        assert_eq!(cel.position.x, 10);
        assert_eq!(cel.position.y, 20);
    }

    #[test]
    fn test_linked_cel() {
        let layer_uuid = Uuid::new_v4();
        let frame_uuid = Uuid::new_v4();
        let cel = Cel::new_linked(layer_uuid, frame_uuid);
        assert!(cel.is_linked());
        assert_eq!(cel.linked_frame_uuid, Some(frame_uuid));
    }

    #[test]
    fn test_cel_validation() {
        let layer_uuid = Uuid::new_v4();
        let cel = Cel::new(layer_uuid, Point::zero(), Rect::new(0, 0, 10, 10));
        assert!(cel.validate().is_ok());

        let mut bad_cel = Cel::new_linked(layer_uuid, Uuid::nil());
        bad_cel.linked_frame_uuid = None;
        assert!(bad_cel.validate().is_err());
    }
}