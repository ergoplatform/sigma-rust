//! ErgoTree
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

#[cfg(test)]
use proptest_derive::Arbitrary;

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 * ErgoTreeSerializer defines top-level serialization format of the scripts.
 */
#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ErgoTree {}

impl SigmaSerializable for ErgoTree {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: W) -> Result<(), io::Error> {
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        Ok(ErgoTree {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ErgoTree>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
