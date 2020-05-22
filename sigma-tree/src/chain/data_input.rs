//! DataInput type

use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

use super::box_id::BoxId;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
/// Inputs, that are used to enrich script context, but won't be spent by the transaction
pub struct DataInput {
    /// id of the box to add into context (should be in UTXO)
    pub box_id: BoxId,
}

impl SigmaSerializable for DataInput {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.box_id.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let box_id = BoxId::sigma_parse(r)?;
        Ok(DataInput { box_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigma_ser::test_helpers::*;

    proptest! {

        #[test]
        fn data_input_roundtrip(v in any::<DataInput>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
