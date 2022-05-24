//! Transitioning type for Base16 encoded bytes in JSON serialization

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
extern crate derive_more;
use derive_more::{From, Into};

use crate::Digest;
use crate::DigestNError;

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

impl<const N: usize> From<Digest<N>> for Base16EncodedBytes {
    fn from(v: Digest<N>) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}

impl<const N: usize> TryFrom<Base16DecodedBytes> for Digest<N> {
    type Error = DigestNError;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let arr: [u8; N] = bytes.0.as_slice().try_into()?;
        Ok(Digest(Box::new(arr)))
    }
}

impl From<&[u8]> for Base16EncodedBytes {
    fn from(v: &[u8]) -> Self {
        Base16EncodedBytes::new(v)
    }
}
