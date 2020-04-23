//! Transactio input
use crate::box_id::BoxId;
use crate::prover_result::ProverResult;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
/// Fully signed transaction input
pub struct Input {
    /// id of the box to spent
    pub box_id: BoxId,
    /// proof of spending correctness
    pub spending_proof: ProverResult,
}

impl SigmaSerializable for Input {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        self.box_id.sigma_serialize(&mut w)?;
        self.spending_proof.sigma_serialize(&mut w)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let box_id = BoxId::sigma_parse(&mut r)?;
        let spending_proof = ProverResult::sigma_parse(&mut r)?;
        Ok(Input {
            box_id,
            spending_proof,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigma_ser::test_helpers::sigma_serialize_roundtrip;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Input>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
