#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

/// Digest size 32 bytes
///
pub const DIGEST32_SIZE: usize = 32;

impl Into<String> for Digest32 {
    fn into(self) -> String {
        base16::encode_lower(&self.0)
    }
}

/// Errors when decoding Digest32 from Base16 encoded string
#[derive(Error, Debug)]
pub enum Digest32DecodeError {
    /// Error on decoding from Base16
    #[error("Base16 decoding error: {0}")]
    Base16DecodeError(base16::DecodeError),
    /// Invalid size of the decoded byte array
    #[error("Invalid byte array size")]
    InvalidSize,
}

impl From<base16::DecodeError> for Digest32DecodeError {
    fn from(e: base16::DecodeError) -> Self {
        Digest32DecodeError::Base16DecodeError(e)
    }
}

impl From<std::array::TryFromSliceError> for Digest32DecodeError {
    fn from(_: std::array::TryFromSliceError) -> Self {
        Digest32DecodeError::InvalidSize
    }
}

/// Generic type for byte array of 32 bytes
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "with-serde", serde(into = "String", try_from = "String"))]
pub struct Digest32(pub [u8; DIGEST32_SIZE]);

impl TryFrom<String> for Digest32 {
    type Error = Digest32DecodeError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = base16::decode(&value)?;
        let arr: [u8; DIGEST32_SIZE] = bytes.as_slice().try_into()?;
        Ok(Digest32(arr))
    }
}

