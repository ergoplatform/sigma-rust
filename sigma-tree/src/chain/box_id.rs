//! Box id type
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

/// Size of Box.id
pub const BOX_ID_SIZE: usize = crate::constants::DIGEST32_SIZE;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
/// newtype for box ids
pub struct BoxId(pub [u8; BOX_ID_SIZE]);

impl SigmaSerializable for BoxId {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_all(&self.0)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let mut bytes = [0; BOX_ID_SIZE];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::test_helpers::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<BoxId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
