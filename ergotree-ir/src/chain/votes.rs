//! Main "remote" type for [Vote]()
use thiserror::Error;

use std::convert::{TryFrom, TryInto};

use super::base16_bytes::Base16DecodedBytes;
use super::base16_bytes::Base16EncodedBytes;

/// Votes for changing system parameters
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
    feature = "json",
    serde(into = "Base16EncodedBytes", try_from = "VotesEncodingVariants")
)]
pub struct Votes(pub [u8; 3]);

#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", serde(untagged))]
#[allow(dead_code)]
enum VotesEncodingVariants {
    AsStr(Base16DecodedBytes),
    /// We need `serde_json::Number` here due to a known `serde_json` bug described here:
    /// <https://github.com/serde-rs/json/issues/740>. Basically we can't deserialise any integer
    /// types directly within untagged enums when the `arbitrary_precision` feature is used. The
    /// workaround is to deserialize as `serde_json::Number` first, then manually convert the type.
    AsByteArray(Vec<serde_json::Number>), // explorer v1
}

impl TryFrom<VotesEncodingVariants> for Votes {
    type Error = VotesError;

    fn try_from(value: VotesEncodingVariants) -> Result<Self, Self::Error> {
        match value {
            VotesEncodingVariants::AsStr(bytes) => bytes.try_into(),
            VotesEncodingVariants::AsByteArray(bytes) => bytes.try_into(),
        }
    }
}

impl From<Votes> for Vec<u8> {
    fn from(v: Votes) -> Self {
        v.0.to_vec()
    }
}

impl TryFrom<Vec<serde_json::Number>> for Votes {
    type Error = VotesError;

    fn try_from(bytes: Vec<serde_json::Number>) -> Result<Self, Self::Error> {
        let bytes_u8: Vec<u8> = bytes
            .into_iter()
            .map(|n| {
                #[allow(clippy::unwrap_used)]
                {
                    n.as_u64().unwrap() as u8
                }
            })
            .collect();
        let arr: [u8; 3] = bytes_u8.as_slice().try_into()?;
        Ok(Self(arr))
    }
}

impl TryFrom<Vec<u8>> for Votes {
    type Error = VotesError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let arr: [u8; 3] = bytes.as_slice().try_into()?;
        Ok(Self(arr))
    }
}

/// Votes errors
#[derive(Error, Debug)]
pub enum VotesError {
    /// Invalid byte array size
    #[error("Votes: Invalid byte array size ({0})")]
    InvalidSize(#[from] std::array::TryFromSliceError),
}

impl TryFrom<Base16DecodedBytes> for Votes {
    type Error = VotesError;

    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        bytes.0.try_into()
    }
}

impl From<Votes> for Base16EncodedBytes {
    fn from(v: Votes) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}
