//! Token related types

use core::fmt;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

/// token id size in bytes
pub const TOKEN_ID_SIZE: usize = crate::constants::DIGEST32_SIZE;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// newtype for token id
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TokenId(pub [u8; TOKEN_ID_SIZE]);

/// Token amount represented with token id paired with it's amount
#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TokenAmount {
    /// token id
    pub token_id: TokenId,
    /// token amount
    pub amount: u64,
}

impl fmt::Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bytes = io::Cursor::new(Vec::new());
        let _ = self.sigma_serialize(&mut bytes);

        f.debug_tuple("TokenId")
            .field(&base16::encode_lower(bytes.get_ref()))
            .finish()
    }
}

impl SigmaSerializable for TokenId {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        w.write_all(&self.0)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let mut bytes = [0; TOKEN_ID_SIZE];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn token_id_roundtrip(v in any::<TokenId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
