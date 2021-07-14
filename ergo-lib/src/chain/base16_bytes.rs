//! Transitioning type for Base16 encoded bytes in JSON serialization

use ergotree_ir::mir::constant::Constant;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
extern crate derive_more;
use derive_more::{From, Into};

/// Transitioning type for Base16 encoded bytes
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(into = "String"))]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct Base16EncodedBytes(String);

impl Base16EncodedBytes {
    /// Create from byte array ref (&[u8])
    pub fn new<T: ?Sized + AsRef<[u8]>>(input: &T) -> Base16EncodedBytes {
        Base16EncodedBytes(base16::encode_lower(input))
    }
}

/// Transitioning type for Base16 decoded bytes
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(try_from = "String", into = "String"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Base16DecodedBytes(pub Vec<u8>);

impl TryFrom<String> for Base16DecodedBytes {
    type Error = base16::DecodeError;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        Ok(Base16DecodedBytes(base16::decode(&str)?))
    }
}

impl From<Base16DecodedBytes> for String {
    fn from(b: Base16DecodedBytes) -> Self {
        base16::encode_lower(&b.0)
    }
}

impl TryFrom<&str> for Base16DecodedBytes {
    type Error = base16::DecodeError;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        Base16DecodedBytes::try_from(v.to_string())
    }
}

impl From<Base16DecodedBytes> for Vec<u8> {
    fn from(b: Base16DecodedBytes) -> Self {
        b.0
    }
}

impl From<Constant> for Base16EncodedBytes {
    fn from(v: Constant) -> Base16EncodedBytes {
        Base16EncodedBytes::new(&v.sigma_serialize_bytes().unwrap())
    }
}

impl TryFrom<Base16DecodedBytes> for Constant {
    type Error = SigmaParsingError;

    fn try_from(value: Base16DecodedBytes) -> Result<Self, Self::Error> {
        Constant::sigma_parse_bytes(&value.0)
    }
}

/// Encodes serialized bytes as Base16
pub trait Base16Str {
    /// Returns serialized bytes encoded as Base16
    fn base16_str(&self) -> String;
}

impl Base16Str for &Constant {
    fn base16_str(&self) -> String {
        base16::encode_lower(&self.sigma_serialize_bytes().unwrap())
    }
}

impl Base16Str for Constant {
    fn base16_str(&self) -> String {
        base16::encode_lower(&self.sigma_serialize_bytes().unwrap())
    }
}
