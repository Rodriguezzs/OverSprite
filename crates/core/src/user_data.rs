//! User data / properties system for OverSprite

use crate::{Point, Rect, Rgba, Size, Uuid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Property value types (matching Aseprite spec)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Fixed(f32),
    Float(f32),
    Double(f64),
    String(String),
    Point(Point),
    Size(Size),
    Rect(Rect),
    Uuid(Uuid),
    Array(Vec<PropertyValue>),
    Object(HashMap<String, PropertyValue>),
}

/// Property map (namespace -> properties)
pub type PropertyMap = HashMap<String, PropertyValue>;

/// User data container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserData {
    pub text: Option<String>,
    pub color: Option<Rgba>,
    pub properties: PropertyMap,
}

// From implementations for PropertyValue
impl From<Vec<u8>> for PropertyValue {
    fn from(v: Vec<u8>) -> Self {
        PropertyValue::Array(v.into_iter().map(PropertyValue::UInt8).collect())
    }
}

impl From<u32> for PropertyValue {
    fn from(v: u32) -> Self {
        PropertyValue::UInt32(v)
    }
}

impl From<u16> for PropertyValue {
    fn from(v: u16) -> Self {
        PropertyValue::UInt16(v)
    }
}

impl From<u8> for PropertyValue {
    fn from(v: u8) -> Self {
        PropertyValue::UInt8(v)
    }
}

impl From<i32> for PropertyValue {
    fn from(v: i32) -> Self {
        PropertyValue::Int32(v)
    }
}

impl From<i16> for PropertyValue {
    fn from(v: i16) -> Self {
        PropertyValue::Int16(v)
    }
}

impl From<i8> for PropertyValue {
    fn from(v: i8) -> Self {
        PropertyValue::Int8(v)
    }
}

impl From<u64> for PropertyValue {
    fn from(v: u64) -> Self {
        PropertyValue::UInt64(v)
    }
}

impl From<i64> for PropertyValue {
    fn from(v: i64) -> Self {
        PropertyValue::Int64(v)
    }
}

impl From<bool> for PropertyValue {
    fn from(v: bool) -> Self {
        PropertyValue::Bool(v)
    }
}

impl From<String> for PropertyValue {
    fn from(v: String) -> Self {
        PropertyValue::String(v)
    }
}

impl From<&str> for PropertyValue {
    fn from(v: &str) -> Self {
        PropertyValue::String(v.to_string())
    }
}

impl UserData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn with_color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.properties.insert(key.into(), value);
    }

    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }

    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    pub fn merge(&mut self, other: UserData) {
        if other.text.is_some() { self.text = other.text; }
        if other.color.is_some() { self.color = other.color; }
        self.properties.extend(other.properties);
    }
}

impl PropertyValue {
    /// Get bytes if this PropertyValue is an Array of UInt8 values
    pub fn as_bytes(&self) -> Option<Vec<u8>> {
        match self {
            PropertyValue::Array(arr) => {
                let mut bytes = Vec::with_capacity(arr.len());
                for val in arr {
                    if let PropertyValue::UInt8(b) = val {
                        bytes.push(*b);
                    } else {
                        return None;
                    }
                }
                Some(bytes)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_data() {
        let mut ud = UserData::new()
            .with_text("hello")
            .with_color(Rgba::from_rgb(255, 0, 0));
        ud.set_property("custom", PropertyValue::String("value".to_string()));
        assert_eq!(ud.text, Some("hello".into()));
        assert_eq!(ud.color, Some(Rgba::from_rgb(255, 0, 0)));
        assert!(ud.has_property("custom"));
    }
}