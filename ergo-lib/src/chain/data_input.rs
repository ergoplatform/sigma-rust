//! DataInput type

use std::io;

use super::ergo_box::{box_id::BoxId, ErgoBox};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SerializationError,
    SigmaSerializable,
};
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Inputs, that are used to enrich script context, but won't be spent by the transaction
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct DataInput {
    /// id of the box to add into context (should be in UTXO)
    #[cfg_attr(feature = "json", serde(rename = "boxId"))]
    pub box_id: BoxId,
}

impl From<&ErgoBox> for DataInput {
    fn from(b: &ErgoBox) -> Self {
        DataInput { box_id: b.box_id() }
    }
}

impl SigmaSerializable for DataInput {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.box_id.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let box_id = BoxId::sigma_parse(r)?;
        Ok(DataInput { box_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;

    proptest! {

        #[test]
        fn data_input_roundtrip(v in any::<DataInput>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
