//! Frame definitions for OverSprite

use crate::{Uuid, UserData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Frame duration in milliseconds
pub type FrameDuration = u32;

/// Frame definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub uuid: Uuid,
    pub duration_ms: FrameDuration,
    pub cels: Vec<Uuid>,  // Ordered list of cel UUIDs in this frame
    pub user_data: UserData,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            duration_ms: 100,
            cels: Vec::new(),
            user_data: UserData::default(),
        }
    }
}

impl Frame {
    pub fn new(duration_ms: FrameDuration) -> Self {
        Self {
            duration_ms,
            ..Default::default()
        }
    }

    pub fn add_cel(&mut self, cel_uuid: Uuid) {
        self.cels.push(cel_uuid);
    }

    pub fn remove_cel(&mut self, cel_uuid: &Uuid) {
        self.cels.retain(|u| u != cel_uuid);
    }

    pub fn cel_count(&self) -> usize {
        self.cels.len()
    }
}

/// Frame index managing frame order
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameIndex {
    pub order: Vec<Uuid>,
    pub default_duration_ms: FrameDuration,
}

impl FrameIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_frame(&mut self, uuid: Uuid) {
        self.order.push(uuid);
    }

    pub fn remove_frame(&mut self, uuid: &Uuid) {
        self.order.retain(|u| u != uuid);
    }

    pub fn move_frame(&mut self, uuid: &Uuid, new_index: usize) {
        if let Some(pos) = self.order.iter().position(|u| u == uuid) {
            let frame = self.order.remove(pos);
            self.order.insert(new_index.min(self.order.len()), frame);
        }
    }

    pub fn frame_at(&self, index: usize) -> Option<&Uuid> {
        self.order.get(index)
    }

    pub fn index_of(&self, uuid: &Uuid) -> Option<usize> {
        self.order.iter().position(|u| u == uuid)
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn total_duration(&self, frames: &HashMap<Uuid, Frame>) -> u64 {
        self.order.iter()
            .filter_map(|uuid| frames.get(uuid))
            .map(|f| f.duration_ms as u64)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let frame = Frame::new(200);
        assert_eq!(frame.duration_ms, 200);
        assert!(frame.cels.is_empty());
    }

    #[test]
    fn test_frame_index() {
        let mut index = FrameIndex::new();
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let uuid3 = Uuid::new_v4();

        index.add_frame(uuid1);
        index.add_frame(uuid2);
        index.add_frame(uuid3);

        assert_eq!(index.len(), 3);
        assert_eq!(index.index_of(&uuid2), Some(1));

        index.move_frame(&uuid3, 0);
        assert_eq!(index.index_of(&uuid3), Some(0));
    }
}