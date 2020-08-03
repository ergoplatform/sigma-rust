use super::op_code::OpCode;
use crate::sigma_protocol::{
    dlog_group::EcPoint, ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree,
};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;

impl SigmaSerializable for SigmaBoolean {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.op_code().sigma_serialize(w)?;
        match self {
            SigmaBoolean::ProofOfKnowledge(proof) => match proof {
                SigmaProofOfKnowledgeTree::ProveDHTuple { .. } => todo!(),
                SigmaProofOfKnowledgeTree::ProveDlog(v) => v.sigma_serialize(w),
            },
            SigmaBoolean::CAND(_) => todo!(),
            SigmaBoolean::TrivialProp(_) => Ok(()), // besides opCode no additional bytes
        }
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let op_code = OpCode::sigma_parse(r)?;
        match op_code {
            OpCode::PROVE_DLOG => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(ProveDlog::sigma_parse(r)?),
            )),
            _ => todo!(),
        }
    }
}

impl SigmaSerializable for ProveDlog {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.h.sigma_serialize(w)
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let p = EcPoint::sigma_parse(r)?;
        Ok(ProveDlog::new(p))
    }
}
