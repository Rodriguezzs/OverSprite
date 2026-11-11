//! Tileset definitions for OverSprite

use crate::{CoreError, CoreResult, Size, Uuid, UserData};
use serde::{Deserialize, Serialize};

// Tileset flags
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct TilesetFlags: u32 {
        const EXTERNAL_FILE = 1;
        const TILES_IN_FILE = 2;
        const EMPTY_TILE_ZERO = 4;
        const AUTO_FLIP_X = 8;
        const AUTO_FLIP_Y = 16;
        const AUTO_FLIP_D = 32;
    }
}

/// Tile data with flip flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileData {
    pub id: u32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_d: bool,
}

impl TileData {
    pub fn new(id: u32) -> Self {
        Self { id, flip_x: false, flip_y: false, flip_d: false }
    }

    pub fn pack(&self, id_mask: u32, flip_x_mask: u32, flip_y_mask: u32, flip_d_mask: u32) -> u32 {
        let mut packed = self.id & id_mask;
        if self.flip_x { packed |= flip_x_mask; }
        if self.flip_y { packed |= flip_y_mask; }
        if self.flip_d { packed |= flip_d_mask; }
        packed
    }

    pub fn unpack(packed: u32, id_mask: u32, flip_x_mask: u32, flip_y_mask: u32, flip_d_mask: u32) -> Self {
        Self {
            id: packed & id_mask,
            flip_x: (packed & flip_x_mask) != 0,
            flip_y: (packed & flip_y_mask) != 0,
            flip_d: (packed & flip_d_mask) != 0,
        }
    }
}

/// Tileset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tileset {
    pub uuid: Uuid,
    pub name: String,
    pub tile_width: u16,
    pub tile_height: u16,
    pub tile_count: u32,
    pub base_index: i16,
    pub flags: TilesetFlags,
    pub external_ref: Option<ExternalTilesetRef>,
    pub image_uuid: Option<Uuid>,  // Reference to cel containing tileset image
    pub tiles: Vec<TileData>,
    pub user_data: UserData,
}

impl Default for Tileset {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "Tileset".to_string(),
            tile_width: 16,
            tile_height: 16,
            tile_count: 0,
            base_index: 1,
            flags: TilesetFlags::EMPTY_TILE_ZERO | TilesetFlags::AUTO_FLIP_X | TilesetFlags::AUTO_FLIP_Y | TilesetFlags::AUTO_FLIP_D,
            external_ref: None,
            image_uuid: None,
            tiles: Vec::new(),
            user_data: UserData::default(),
        }
    }
}

impl Tileset {
    pub fn new(name: impl Into<String>, tile_width: u16, tile_height: u16) -> Self {
        Self {
            name: name.into(),
            tile_width,
            tile_height,
            ..Default::default()
        }
    }

    pub fn tile_size(&self) -> Size {
        Size::new(self.tile_width as u32, self.tile_height as u32)
    }

    pub fn add_tile(&mut self, tile: TileData) {
        self.tiles.push(tile);
        self.tile_count = self.tiles.len() as u32;
    }

    pub fn get_tile(&self, index: usize) -> Option<&TileData> {
        self.tiles.get(index)
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.name.is_empty() {
            return Err(CoreError::ValidationError("Tileset name cannot be empty".into()));
        }
        if self.tile_width == 0 || self.tile_height == 0 {
            return Err(CoreError::ValidationError("Tile dimensions must be non-zero".into()));
        }
        Ok(())
    }
}

/// External tileset reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalTilesetRef {
    pub file_id: Uuid,
    pub tileset_id: Uuid,
}

/// Tilemap layer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tilemap {
    pub width_tiles: u16,
    pub height_tiles: u16,
    pub bpp: u8,  // 8, 16, or 32
    pub id_mask: u32,
    pub flip_x_mask: u32,
    pub flip_y_mask: u32,
    pub flip_d_mask: u32,
    pub tiles: Vec<u32>,  // Packed tile data
}

impl Tilemap {
    pub fn new(width: u16, height: u16, bpp: u8) -> Self {
        let (id_mask, flip_x_mask, flip_y_mask, flip_d_mask) = match bpp {
            8 => (0xFF, 0, 0, 0),
            16 => (0x3FFF, 0x4000, 0x8000, 0),
            32 => (0x1FFFFFFF, 0x20000000, 0x40000000, 0x80000000),
            _ => (0x1FFFFFFF, 0x20000000, 0x40000000, 0x80000000),
        };
        Self {
            width_tiles: width,
            height_tiles: height,
            bpp,
            id_mask,
            flip_x_mask,
            flip_y_mask,
            flip_d_mask,
            tiles: vec![0; (width * height) as usize],
        }
    }

    pub fn set_tile(&mut self, x: u16, y: u16, tile: TileData) {
        if x < self.width_tiles && y < self.height_tiles {
            let idx = (y * self.width_tiles + x) as usize;
            self.tiles[idx] = tile.pack(self.id_mask, self.flip_x_mask, self.flip_y_mask, self.flip_d_mask);
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> Option<TileData> {
        if x < self.width_tiles && y < self.height_tiles {
            let idx = (y * self.width_tiles + x) as usize;
            Some(TileData::unpack(self.tiles[idx], self.id_mask, self.flip_x_mask, self.flip_y_mask, self.flip_d_mask))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tileset_creation() {
        let ts = Tileset::new("Test", 16, 16);
        assert_eq!(ts.tile_width, 16);
        assert_eq!(ts.tile_height, 16);
    }

    #[test]
    fn test_tile_packing() {
        let tile = TileData::new(42);
        let packed = tile.pack(0x1FFFFFFF, 0x20000000, 0x40000000, 0x80000000);
        assert_eq!(packed & 0x1FFFFFFF, 42);

        let unpacked = TileData::unpack(packed, 0x1FFFFFFF, 0x20000000, 0x40000000, 0x80000000);
        assert_eq!(unpacked.id, 42);
    }

    #[test]
    fn test_tilemap() {
        let mut tm = Tilemap::new(10, 10, 32);
        tm.set_tile(5, 5, TileData::new(42));
        let tile = tm.get_tile(5, 5).unwrap();
        assert_eq!(tile.id, 42);
    }
}