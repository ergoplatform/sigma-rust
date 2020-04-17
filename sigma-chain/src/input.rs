use crate::box_id::BoxId;
use crate::prover_result::ProverResult;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

pub struct Input {
    pub box_id: BoxId,
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
