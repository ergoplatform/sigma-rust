//! Transitioning type for Base16 encoded bytes in JSON serialization

use core::fmt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Transitioning type for Base16 encoded bytes
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "with-serde", serde(into = "String"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Base16EncodedBytes(String);

impl Base16EncodedBytes {
    /// Create from byte array ref (&[u8])
    pub fn new<T: ?Sized + AsRef<[u8]>>(input: &T) -> Base16EncodedBytes {
        Base16EncodedBytes(base16::encode_lower(input))
    }
}

impl Into<String> for Base16EncodedBytes {
    fn into(self) -> String {
        self.0
    }
}

impl fmt::Display for Base16EncodedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TokenId").field(&self.0).finish()
    }
}

/// Transitioning type for Base16 decoded bytes
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "with-serde", serde(try_from = "String"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Base16DecodedBytes(pub Vec<u8>);

impl TryFrom<String> for Base16DecodedBytes {
    type Error = base16::DecodeError;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        Ok(Base16DecodedBytes(base16::decode(&str)?))
    }
}
