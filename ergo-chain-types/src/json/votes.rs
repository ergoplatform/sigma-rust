//! Code to implement `Votes` JSON encoding

use alloc::string::ToString;
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};

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
                // Each element is untrusted JSON: it must be an integer that
                // fits in a byte. Anything else (negative, fractional, above
                // 255, or beyond u64) is a parse error, never a panic or a
                // silently truncated value.
                n.as_u64()
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or_else(|| VotesError::InvalidByteValue(n.to_string()))
            })
            .collect::<Result<Vec<u8>, VotesError>>()?;
        let arr: [u8; 3] = bytes_u8.as_slice().try_into()?;
        Ok(Self(arr))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::votes::Votes;

    #[test]
    fn parse_byte_array_votes() {
        let votes: Votes = serde_json::from_str("[4, 0, 255]").unwrap();
        assert_eq!(votes, Votes([4, 0, 255]));
    }

    #[test]
    fn parse_hex_string_votes() {
        let votes: Votes = serde_json::from_str("\"0400ff\"").unwrap();
        assert_eq!(votes, Votes([4, 0, 255]));
    }

    #[test]
    fn reject_negative_vote_byte() {
        // A JSON number is untrusted input: a negative element must produce a
        // parse error, not a panic.
        assert!(serde_json::from_str::<Votes>("[-1, 0, 0]").is_err());
    }

    #[test]
    fn reject_fractional_vote_byte() {
        assert!(serde_json::from_str::<Votes>("[1.5, 0, 0]").is_err());
    }

    #[test]
    fn reject_vote_byte_above_255() {
        // 256 must not be silently truncated to 0 by an `as u8` cast.
        assert!(serde_json::from_str::<Votes>("[256, 0, 0]").is_err());
    }

    #[test]
    fn reject_huge_vote_byte() {
        assert!(serde_json::from_str::<Votes>("[18446744073709551616, 0, 0]").is_err());
    }

    #[test]
    fn reject_wrong_number_of_votes() {
        assert!(serde_json::from_str::<Votes>("[0, 0]").is_err());
        assert!(serde_json::from_str::<Votes>("[0, 0, 0, 0]").is_err());
    }
}
