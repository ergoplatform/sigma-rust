//! Token related types

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SerializationError,
    SigmaSerializable,
};
use std::convert::TryFrom;
use std::io;

use super::digest32::Digest32;
use super::ergo_box::box_id::BoxId;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// newtype for token id
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct TokenId(pub Digest32);

impl TokenId {
    /// token id size in bytes
    pub const SIZE: usize = Digest32::SIZE;
}

impl From<BoxId> for TokenId {
    fn from(i: BoxId) -> Self {
        TokenId(i.0)
    }
}

impl SigmaSerializable for TokenId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.0.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self(Digest32::sigma_parse(r)?))
    }
}

/// Box value with bound checks
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct TokenAmount(u64);

impl TokenAmount {
    /// minimal allowed value
    pub const MIN: u64 = 1;
    /// maximal allowed value
    pub const MAX: u64 = i64::MAX as u64;
}

/// BoxValue errors
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum TokenAmountError {
    /// Value is out of bounds
    #[error("Value is out of bounds")]
    OutOfBounds,
    /// Overflow
    #[error("Overflow")]
    Overflow,
}

impl TryFrom<u64> for TokenAmount {
    type Error = TokenAmountError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        if v >= TokenAmount::MIN && v <= TokenAmount::MAX {
            Ok(TokenAmount(v))
        } else {
            Err(TokenAmountError::OutOfBounds)
        }
    }
}

impl From<TokenAmount> for u64 {
    fn from(ta: TokenAmount) -> Self {
        ta.0
    }
}

/// Token represented with token id paired with it's amount
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Token {
    /// token id
    #[cfg_attr(feature = "json", serde(rename = "tokenId"))]
    pub token_id: TokenId,
    /// token amount
    #[cfg_attr(feature = "json", serde(rename = "amount"))]
    pub amount: TokenAmount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    impl Arbitrary for TokenAmount {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (TokenAmount::MIN..=TokenAmount::MAX).prop_map(Self).boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {

        #[test]
        fn token_id_roundtrip(v in any::<TokenId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
