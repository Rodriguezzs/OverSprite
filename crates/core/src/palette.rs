//! Palette definitions for OverSprite

use crate::{Rgba, Uuid, UserData, CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Palette color entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub name: Option<String>,
}

impl PaletteColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a, name: None }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn to_rgba(&self) -> Rgba {
        Rgba::new(self.r, self.g, self.b, self.a)
    }

    pub fn from_rgba(rgba: Rgba) -> Self {
        Self::new(rgba.r, rgba.g, rgba.b, rgba.a)
    }
}

impl Default for PaletteColor {
    fn default() -> Self {
        Self::new(0, 0, 0, 255)
    }
}

impl From<&PaletteColor> for Rgba {
    fn from(color: &PaletteColor) -> Self {
        color.to_rgba()
    }
}

impl From<PaletteColor> for Rgba {
    fn from(color: PaletteColor) -> Self {
        color.to_rgba()
    }
}

/// Palette index type
pub type PaletteIndex = u16;

/// Color palette for indexed color mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub uuid: Uuid,
    pub name: String,
    pub colors: Vec<PaletteColor>,
    pub color_names: HashMap<PaletteIndex, String>,
    pub user_data: UserData,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "Default".to_string(),
            colors: vec![PaletteColor::default(); 256],
            color_names: HashMap::new(),
            user_data: UserData::default(),
        }
    }
}

impl Palette {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_colors(name: impl Into<String>, colors: Vec<PaletteColor>) -> Self {
        let mut palette = Self::new(name);
        palette.colors = colors;
        palette.rebuild_names();
        palette
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn get(&self, index: PaletteIndex) -> Option<&PaletteColor> {
        self.colors.get(index as usize)
    }

    pub fn get_mut(&mut self, index: PaletteIndex) -> Option<&mut PaletteColor> {
        self.colors.get_mut(index as usize)
    }

    pub fn set(&mut self, index: PaletteIndex, color: PaletteColor) -> CoreResult<()> {
        if index as usize >= self.colors.len() {
            return Err(CoreError::PaletteIndexOutOfBounds { index: index as usize, max: self.colors.len() });
        }
        self.colors[index as usize] = color;
        self.rebuild_names();
        Ok(())
    }

    pub fn push(&mut self, color: PaletteColor) {
        self.colors.push(color);
        self.rebuild_names();
    }

    pub fn resize(&mut self, new_size: usize) {
        self.colors.resize(new_size, PaletteColor::default());
        self.rebuild_names();
    }

    pub fn find_color(&self, rgba: Rgba) -> Option<PaletteIndex> {
        self.colors.iter().position(|c| c.to_rgba() == rgba).map(|i| i as PaletteIndex)
    }

    pub fn find_nearest(&self, rgba: Rgba) -> PaletteIndex {
        let mut best_idx = 0;
        let mut best_dist = u32::MAX;

        for (i, color) in self.colors.iter().enumerate() {
            let c = color.to_rgba();
            let dr = c.r as i32 - rgba.r as i32;
            let dg = c.g as i32 - rgba.g as i32;
            let db = c.b as i32 - rgba.b as i32;
            let da = c.a as i32 - rgba.a as i32;
            let dist = (dr * dr + dg * dg + db * db + da * da) as u32;

            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }

        best_idx as PaletteIndex
    }

    fn rebuild_names(&mut self) {
        self.color_names.clear();
        for (i, color) in self.colors.iter().enumerate() {
            if let Some(name) = &color.name {
                self.color_names.insert(i as PaletteIndex, name.clone());
            }
        }
    }

    pub fn to_rgba_array(&self) -> Vec<Rgba> {
        self.colors.iter().map(|c| c.to_rgba()).collect()
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.name.is_empty() {
            return Err(CoreError::ValidationError("Palette name cannot be empty".to_string()));
        }
        if self.colors.is_empty() {
            return Err(CoreError::ValidationError("Palette must have at least one color".to_string()));
        }
        if self.colors.len() > 65536 {
            return Err(CoreError::ValidationError("Palette cannot exceed 65536 colors".to_string()));
        }
        Ok(())
    }
}

/// Per-frame palette override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramePaletteOverride {
    pub base_palette_uuid: Uuid,
    pub changes: HashMap<PaletteIndex, PaletteColor>,
}

impl FramePaletteOverride {
    pub fn new(base_palette_uuid: Uuid) -> Self {
        Self {
            base_palette_uuid,
            changes: HashMap::new(),
        }
    }

    pub fn apply(&self, base: &Palette) -> Palette {
        let mut result = base.clone();
        for (index, color) in &self.changes {
            if let Some(entry) = result.get_mut(*index) {
                *entry = color.clone();
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_creation() {
        let palette = Palette::new("Test");
        assert_eq!(palette.name, "Test");
        assert_eq!(palette.len(), 256);
    }

    #[test]
    fn test_palette_color_ops() {
        let mut palette = Palette::new("Test");
        let color = PaletteColor::new(255, 0, 0, 255).with_name("red");
        palette.set(0, color).unwrap();
        assert_eq!(palette.get(0).unwrap().to_rgba(), Rgba::new(255, 0, 0, 255));
    }

    #[test]
    fn test_find_nearest() {
        let mut palette = Palette::new("Test");
        palette.set(0, PaletteColor::new(0, 0, 0, 255)).unwrap();
        palette.set(1, PaletteColor::new(255, 255, 255, 255)).unwrap();
        palette.set(2, PaletteColor::new(255, 0, 0, 255)).unwrap();

        let idx = palette.find_nearest(Rgba::new(250, 10, 10, 255));
        assert_eq!(idx, 2); // Should find red
    }
}