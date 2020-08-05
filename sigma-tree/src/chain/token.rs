//! Token related types

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use sigma_ser::vlq_encode;
use std::io;

use super::digest32::Digest32;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

/// newtype for token id
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TokenId(pub Digest32);

impl TokenId {
    /// token id size in bytes
    pub const SIZE: usize = Digest32::SIZE;
}

impl SigmaSerializable for TokenId {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.0.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self(Digest32::sigma_parse(r)?))
    }
}

/// Token amount represented with token id paired with it's amount
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TokenAmount {
    /// token id
    #[cfg_attr(feature = "with-serde", serde(rename = "tokenId"))]
    pub token_id: TokenId,
    /// token amount
    #[cfg_attr(feature = "with-serde", serde(rename = "amount"))]
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn token_id_roundtrip(v in any::<TokenId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
