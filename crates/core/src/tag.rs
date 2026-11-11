//! Animation tag definitions for OverSprite

use crate::{AnimationDirection, CoreError, CoreResult, Rgba, Uuid, UserData};
use serde::{Deserialize, Serialize};

/// Animation tag (named frame range)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub uuid: Uuid,
    pub name: String,
    pub from_frame: u32,
    pub to_frame: u32,
    pub direction: AnimationDirection,
    pub repeat: u16,
    pub color: Rgba,
    pub user_data: UserData,
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "Tag".to_string(),
            from_frame: 0,
            to_frame: 0,
            direction: AnimationDirection::Forward,
            repeat: 0,
            color: Rgba::new(255, 100, 100, 255),
            user_data: UserData::default(),
        }
    }
}

impl Tag {
    pub fn new(name: impl Into<String>, from: u32, to: u32) -> Self {
        Self {
            name: name.into(),
            from_frame: from,
            to_frame: to,
            ..Default::default()
        }
    }

    pub fn frame_count(&self) -> u32 {
        if self.to_frame >= self.from_frame {
            self.to_frame - self.from_frame + 1
        } else {
            0
        }
    }

    pub fn validate(&self, total_frames: u32) -> CoreResult<()> {
        if self.name.is_empty() {
            return Err(CoreError::ValidationError("Tag name cannot be empty".into()));
        }
        if self.from_frame >= total_frames || self.to_frame >= total_frames {
            return Err(CoreError::ValidationError("Tag frame range exceeds total frames".into()));
        }
        if self.from_frame > self.to_frame {
            return Err(CoreError::ValidationError("Tag from_frame must be <= to_frame".into()));
        }
        Ok(())
    }

    pub fn contains_frame(&self, frame: u32) -> bool {
        frame >= self.from_frame && frame <= self.to_frame
    }

    pub fn frames_in_order(&self) -> Vec<u32> {
        match self.direction {
            AnimationDirection::Forward => (self.from_frame..=self.to_frame).collect(),
            AnimationDirection::Reverse => (self.from_frame..=self.to_frame).rev().collect(),
            AnimationDirection::PingPong => {
                let mut frames: Vec<u32> = (self.from_frame..=self.to_frame).collect();
                frames.extend((self.from_frame + 1..self.to_frame).rev());
                frames
            }
            AnimationDirection::PingPongReverse => {
                let mut frames: Vec<u32> = (self.from_frame..=self.to_frame).rev().collect();
                frames.extend(self.from_frame + 1..self.to_frame);
                frames
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("walk", 0, 7);
        assert_eq!(tag.name, "walk");
        assert_eq!(tag.frame_count(), 8);
    }

    #[test]
    fn test_tag_frames() {
        let tag = Tag::new("walk", 0, 3);
        assert_eq!(tag.frames_in_order(), vec![0, 1, 2, 3]);

        let tag = Tag { direction: AnimationDirection::Reverse, ..tag };
        assert_eq!(tag.frames_in_order(), vec![3, 2, 1, 0]);

        let tag = Tag { direction: AnimationDirection::PingPong, ..Tag::new("walk", 0, 3) };
        assert_eq!(tag.frames_in_order(), vec![0, 1, 2, 3, 2, 1]);
    }
}