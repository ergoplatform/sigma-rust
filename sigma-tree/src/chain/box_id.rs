//! Box id type
use super::digest32::{Digest32, DIGEST32_SIZE};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// newtype for box ids
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "with-serde", serde(into = "Digest32", from = "Digest32"))]
#[cfg_attr(test, derive(Arbitrary))]
pub struct BoxId(pub [u8; BoxId::SIZE]);

impl BoxId {
    /// Size in bytes
    pub const SIZE: usize = DIGEST32_SIZE;

    /// All zeros
    pub fn zero() -> BoxId {
        BoxId([0u8; BoxId::SIZE])
    }
}

// TODO: try custom derive?
impl From<Digest32> for BoxId {
    fn from(d: Digest32) -> Self {
        BoxId(d.0)
    }
}
impl From<BoxId> for Digest32 {
    fn from(d: BoxId) -> Self {
        Digest32(d.0)
    }
}

impl SigmaSerializable for BoxId {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_all(&self.0)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let mut bytes = [0; DIGEST32_SIZE];
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
