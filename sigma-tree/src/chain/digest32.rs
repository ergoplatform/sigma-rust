use std::convert::TryInto;
use thiserror::Error;

/// Digest size 32 bytes
pub const DIGEST32_SIZE: usize = 32;

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

pub fn decode_base16(value: String) -> Result<[u8; DIGEST32_SIZE], Digest32DecodeError> {
    let bytes = base16::decode(&value)?;
    let arr: [u8; DIGEST32_SIZE] = bytes.as_slice().try_into()?;
    Ok(arr)
}

pub fn encode_base16(value: &[u8; DIGEST32_SIZE]) -> String {
    base16::encode_lower(value)
}
