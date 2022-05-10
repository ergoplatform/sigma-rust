//! Code to implement `Votes` JSON encoding

use std::convert::{TryFrom, TryInto};

use crate::votes::{Votes, VotesError};
use crate::Base16DecodedBytes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum VotesEncodingVariants {
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
