//! Grid definitions for OverSprite

use crate::{Point, Size};
use serde::{Deserialize, Serialize};

/// Grid configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub enabled: bool,
    pub origin: Point,
    pub cell_size: Size,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            enabled: false,
            origin: Point::zero(),
            cell_size: Size::new(16, 16),
        }
    }
}

impl Grid {
    pub fn new(origin: Point, cell_size: Size) -> Self {
        Self { enabled: true, origin, cell_size }
    }

    pub fn snap(&self, point: Point) -> Point {
        if !self.enabled || self.cell_size.w == 0 || self.cell_size.h == 0 {
            return point;
        }
        let x = self.origin.x + ((point.x - self.origin.x) / self.cell_size.w as i32) * self.cell_size.w as i32;
        let y = self.origin.y + ((point.y - self.origin.y) / self.cell_size.h as i32) * self.cell_size.h as i32;
        Point::new(x, y)
    }
}