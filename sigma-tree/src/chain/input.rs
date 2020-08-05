//! Transaction input
use sigma_ser::vlq_encode;
use std::io;

use super::{box_id::BoxId, prover_result::ProverResult};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

/// Fully signed transaction input
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Input {
    /// id of the box to spent
    #[cfg_attr(feature = "with-serde", serde(rename = "boxId"))]
    pub box_id: BoxId,
    /// proof of spending correctness
    #[cfg_attr(feature = "with-serde", serde(rename = "spendingProof"))]
    pub spending_proof: ProverResult,
}

impl Input {
    /// input with an empty proof
    pub fn input_to_sign(&self) -> Input {
        Input {
            box_id: self.box_id.clone(),
            spending_proof: ProverResult {
                proof: vec![],
                extension: self.spending_proof.extension.clone(),
            },
        }
    }
}

impl SigmaSerializable for Input {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.box_id.sigma_serialize(w)?;
        self.spending_proof.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let box_id = BoxId::sigma_parse(r)?;
        let spending_proof = ProverResult::sigma_parse(r)?;
        Ok(Input {
            box_id,
            spending_proof,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Input>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
