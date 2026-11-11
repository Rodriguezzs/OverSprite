//! UUID extension for OverSprite

use uuid::Uuid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Wrapper for UUID with custom serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UuidExt(pub Uuid);

impl UuidExt {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes(bytes))
    }

    pub fn to_bytes(self) -> [u8; 16] {
        self.0.into_bytes()
    }
}

impl Default for UuidExt {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for UuidExt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for UuidExt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map(Self).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for UuidExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for UuidExt {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for UuidExt {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<UuidExt> for Uuid {
    fn from(u: UuidExt) -> Self {
        u.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_ext() {
        let uuid = UuidExt::new();
        assert!(!uuid.is_nil());

        let nil = UuidExt::nil();
        assert!(nil.is_nil());
    }

    #[test]
    fn test_uuid_serialization() {
        let uuid = UuidExt::new();
        let json = serde_json::to_string(&uuid).unwrap();
        let deserialized: UuidExt = serde_json::from_str(&json).unwrap();
        assert_eq!(uuid, deserialized);
    }
}