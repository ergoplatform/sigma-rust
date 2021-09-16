//! DataInput type

use crate::chain::ergo_box::BoxId;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializeResult;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Inputs, that are used to enrich script context, but won't be spent by the transaction
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct DataInput {
    /// id of the box to add into context (should be in UTXO)
    #[cfg_attr(feature = "json", serde(rename = "boxId", alias = "id"))]
    pub box_id: BoxId,
}

impl From<BoxId> for DataInput {
    fn from(box_id: BoxId) -> Self {
        DataInput { box_id }
    }
}

impl SigmaSerializable for DataInput {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.box_id.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let box_id = BoxId::sigma_parse(r)?;
        Ok(DataInput { box_id })
    }
}

#[cfg(test)]
mod tests {
    use ergotree_ir::serialization::sigma_serialize_roundtrip;

    use super::*;

    proptest! {

        #[test]
        fn data_input_roundtrip(v in any::<DataInput>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
