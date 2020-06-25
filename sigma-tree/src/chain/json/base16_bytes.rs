//! Transitioning type for Base16 encoded bytes in JSON serialization

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Transitioning type for Base16 encoded bytes
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "with-serde", serde(into = "String", try_from = "String"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Base16Bytes(pub Vec<u8>);

impl Into<String> for Base16Bytes {
    fn into(self) -> String {
        base16::encode_lower(&self.0)
    }
}

impl TryFrom<String> for Base16Bytes {
    type Error = base16::DecodeError;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        Ok(Base16Bytes(base16::decode(&str)?))
    }
}
