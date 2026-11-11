//! Slice definitions for OverSprite (9-patch, export slices)

use crate::{CoreError, CoreResult, Point, Rect, Size, Uuid, UserData};
use serde::{Deserialize, Serialize};

/// Slice key (per-frame slice data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceKey {
    pub frame: u32,
    pub bounds: Rect,
    pub is_9patch: bool,
    pub center: Option<Rect>,
    pub pivot: Option<Point>,
    pub hidden: bool,
}

impl SliceKey {
    pub fn new(frame: u32, bounds: Rect) -> Self {
        Self {
            frame,
            bounds,
            is_9patch: false,
            center: None,
            pivot: None,
            hidden: false,
        }
    }
}

/// Slice definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice {
    pub uuid: Uuid,
    pub name: String,
    pub keys: Vec<SliceKey>,
    pub is_9patch: bool,
    pub has_pivot: bool,
    pub user_data: UserData,
}

impl Default for Slice {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "Slice".to_string(),
            keys: Vec::new(),
            is_9patch: false,
            has_pivot: false,
            user_data: UserData::default(),
        }
    }
}

impl Slice {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Default::default() }
    }

    pub fn add_key(&mut self, key: SliceKey) {
        self.keys.push(key);
    }

    pub fn key_at_frame(&self, frame: u32) -> Option<&SliceKey> {
        self.keys.iter().rev().find(|k| k.frame <= frame && !k.hidden)
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.name.is_empty() {
            return Err(CoreError::ValidationError("Slice name cannot be empty".into()));
        }
        Ok(())
    }
}

/// 9-patch slice data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice9Patch {
    pub center: Rect,
    pub stretch_x: Vec<u32>,
    pub stretch_y: Vec<u32>,
}

/// Pivot point
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pivot {
    pub x: i32,
    pub y: i32,
}

impl Pivot {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn center(size: Size) -> Self {
        Self::new(size.w as i32 / 2, size.h as i32 / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_creation() {
        let slice = Slice::new("button");
        assert_eq!(slice.name, "button");
    }

    #[test]
    fn test_pivot_center() {
        let size = Size::new(100, 100);
        let pivot = Pivot::center(size);
        assert_eq!(pivot.x, 50);
        assert_eq!(pivot.y, 50);
    }
}