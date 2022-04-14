//! Token related types

use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SigmaParsingError,
    SigmaSerializable,
};
use std::convert::TryFrom;

use super::ergo_box::BoxId;
use derive_more::From;
use derive_more::Into;
use ergo_chain_types::{Digest32, DigestNError};
use sigma_ser::ScorexSerializable;
use thiserror::Error;

/// newtype for token id
#[derive(PartialEq, Eq, Hash, Debug, Clone, From, Into)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenId(Digest32);

impl TokenId {
    /// token id size in bytes
    pub const SIZE: usize = Digest32::SIZE;

    /// Parse TokenId from base64 encoded string
    pub fn from_base64(s: &str) -> Result<TokenId, DigestNError> {
        Digest32::from_base64(s).map(Into::into)
    }
}

impl From<BoxId> for TokenId {
    fn from(i: BoxId) -> Self {
        TokenId(i.into())
    }
}

impl From<TokenId> for Vec<i8> {
    fn from(v: TokenId) -> Self {
        v.0.into()
    }
}

impl From<TokenId> for Vec<u8> {
    fn from(v: TokenId) -> Self {
        v.0.into()
    }
}

impl AsRef<[u8]> for TokenId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<TokenId> for String {
    fn from(v: TokenId) -> Self {
        v.0.into()
    }
}

impl SigmaSerializable for TokenId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        Ok(self.0.scorex_serialize(w)?)
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        Ok(Self(Digest32::scorex_parse(r)?))
    }
}

/// Token amount with bound checks
#[cfg(not(feature = "json"))]
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct TokenAmount(u64);

/// Token amount with bound checks
#[cfg(feature = "json")]
#[derive(
    serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord,
)]
#[serde(
    try_from = "crate::chain::json::token::TokenAmountJson",
    into = "crate::chain::json::token::TokenAmountJson"
)]
pub struct TokenAmount(pub(crate) u64);

impl TokenAmount {
    /// minimal allowed value
    pub const MIN_RAW: u64 = 1;
    /// maximal allowed value
    pub const MAX_RAW: u64 = i64::MAX as u64;

    /// Addition with overflow check
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, TokenAmountError> {
        let raw = self
            .0
            .checked_add(rhs.0)
            .ok_or(TokenAmountError::Overflow)?;
        if raw > Self::MAX_RAW {
            Err(TokenAmountError::OutOfBounds(raw))
        } else {
            Ok(Self(raw))
        }
    }

    /// Subtraction with overflow and bounds check
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, TokenAmountError> {
        let raw = self
            .0
            .checked_sub(rhs.0)
            .ok_or(TokenAmountError::Overflow)?;
        if raw < Self::MIN_RAW {
            Err(TokenAmountError::OutOfBounds(raw))
        } else {
            Ok(Self(raw))
        }
    }

    /// Get the value as u64
    pub fn as_u64(&self) -> &u64 {
        &self.0
    }
}

/// BoxValue errors
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum TokenAmountError {
    /// Value is out of bounds
    #[error("Token amount is out of bounds: {0}")]
    OutOfBounds(u64),
    /// Overflow
    #[error("Overflow")]
    Overflow,
}

impl TryFrom<u64> for TokenAmount {
    type Error = TokenAmountError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        if (TokenAmount::MIN_RAW..=TokenAmount::MAX_RAW).contains(&v) {
            Ok(TokenAmount(v))
        } else {
            Err(TokenAmountError::OutOfBounds(v))
        }
    }
}

impl From<TokenAmount> for u64 {
    fn from(ta: TokenAmount) -> Self {
        ta.0
    }
}

impl From<TokenAmount> for i64 {
    fn from(ta: TokenAmount) -> Self {
        ta.0 as i64
    }
}

impl From<Token> for (Vec<i8>, i64) {
    fn from(t: Token) -> Self {
        (t.token_id.into(), t.amount.into())
    }
}

/// Token represented with token id paired with it's amount
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct Token {
    /// token id
    #[cfg_attr(feature = "json", serde(rename = "tokenId"))]
    pub token_id: TokenId,
    /// token amount
    #[cfg_attr(feature = "json", serde(rename = "amount"))]
    pub amount: TokenAmount,
}

impl From<(TokenId, TokenAmount)> for Token {
    fn from(token_pair: (TokenId, TokenAmount)) -> Self {
        Token {
            token_id: token_pair.0,
            amount: token_pair.1,
        }
    }
}

/// Arbitrary
#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use ergo_chain_types::Digest32;
    use proptest::prelude::*;

    use super::TokenAmount;
    use super::TokenId;

    use std::convert::TryFrom;

    /// How to generate a token id
    pub enum ArbTokenIdParam {
        /// From a list of predefined token ids
        Predef,
        /// Arbitrary token id
        Arbitrary,
    }

    impl Default for ArbTokenIdParam {
        fn default() -> Self {
            ArbTokenIdParam::Predef
        }
    }

    impl Arbitrary for TokenId {
        type Parameters = ArbTokenIdParam;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            match args {
                ArbTokenIdParam::Predef => prop_oneof![
                    Just(TokenId::from(
                        Digest32::try_from(
                            "3130a82e45842aebb888742868e055e2f554ab7d92f233f2c828ed4a43793710"
                                .to_string()
                        )
                        .unwrap()
                    )),
                    Just(TokenId::from(
                        Digest32::try_from(
                            "e7321ffb4ec5d71deb3110eb1ac09612b9cf57445acab1e0e3b1222d5b5a6c60"
                                .to_string()
                        )
                        .unwrap()
                    )),
                    Just(TokenId::from(
                        Digest32::try_from(
                            "ad62f6dd92e7dc850bc406770dfac9a943dd221a7fb440b7b2bcc7d3149c1792"
                                .to_string()
                        )
                        .unwrap()
                    ))
                ]
                .boxed(),
                ArbTokenIdParam::Arbitrary => (any::<Digest32>()).prop_map_into().boxed(),
            }
        }

        type Strategy = BoxedStrategy<Self>;
    }

    impl Arbitrary for TokenAmount {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (TokenAmount::MIN_RAW..=TokenAmount::MAX_RAW / 100000)
                .prop_map(Self)
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {

    use crate::chain::token::TokenId;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn token_id_roundtrip(v in any::<TokenId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
