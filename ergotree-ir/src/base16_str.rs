//! Trait for base16-encoded serialized bytes

use crate::serialization::SigmaSerializationError;

/// Encodes serialized bytes as Base16
pub trait Base16Str {
    /// Returns serialized bytes encoded as Base16
    fn base16_str(&self) -> Result<String, SigmaSerializationError>;
}

