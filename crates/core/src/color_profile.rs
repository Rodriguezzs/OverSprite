//! Color profile definitions for OverSprite

use serde::{Deserialize, Serialize};

/// Color profile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorProfileType {
    None,
    Srgb,
    Icc,
}

/// Color profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorProfile {
    pub profile_type: ColorProfileType,
    pub fixed_gamma: f32,
    pub use_fixed_gamma: bool,
    pub icc_data: Option<Vec<u8>>,
}

impl Default for ColorProfile {
    fn default() -> Self {
        Self::srgb()
    }
}

impl ColorProfile {
    pub fn none() -> Self {
        Self {
            profile_type: ColorProfileType::None,
            fixed_gamma: 1.0,
            use_fixed_gamma: false,
            icc_data: None,
        }
    }

    pub fn srgb() -> Self {
        Self {
            profile_type: ColorProfileType::Srgb,
            fixed_gamma: 2.2,
            use_fixed_gamma: false,
            icc_data: None,
        }
    }

    pub fn linear_srgb() -> Self {
        Self {
            profile_type: ColorProfileType::Srgb,
            fixed_gamma: 1.0,
            use_fixed_gamma: true,
            icc_data: None,
        }
    }

    pub fn icc(data: Vec<u8>) -> Self {
        Self {
            profile_type: ColorProfileType::Icc,
            fixed_gamma: 1.0,
            use_fixed_gamma: false,
            icc_data: Some(data),
        }
    }

    pub fn to_gamma(&self, linear: f32) -> f32 {
        if self.use_fixed_gamma {
            linear.powf(1.0 / self.fixed_gamma)
        } else if self.profile_type == ColorProfileType::Srgb {
            // sRGB gamma curve
            if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            }
        } else {
            linear
        }
    }

    pub fn from_gamma(&self, gamma: f32) -> f32 {
        if self.use_fixed_gamma {
            gamma.powf(self.fixed_gamma)
        } else if self.profile_type == ColorProfileType::Srgb {
            // Inverse sRGB gamma curve
            if gamma <= 0.04045 {
                gamma / 12.92
            } else {
                ((gamma + 0.055) / 1.055).powf(2.4)
            }
        } else {
            gamma
        }
    }
}