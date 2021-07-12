use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::has_opcode::HasOpCode;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use crate::sigma_protocol::{
    dlog_group::EcPoint,
    sigma_boolean::{
        ProveDhTuple, ProveDlog, SigmaBoolean, SigmaConjecture, SigmaProofOfKnowledgeTree,
    },
};

use crate::sigma_protocol::sigma_boolean::cand::Cand;
use crate::sigma_protocol::sigma_boolean::cor::Cor;
use crate::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
use std::io;

impl SigmaSerializable for SigmaBoolean {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.op_code().sigma_serialize(w)?;
        match self {
            SigmaBoolean::ProofOfKnowledge(proof) => match proof {
                SigmaProofOfKnowledgeTree::ProveDhTuple(v) => v.sigma_serialize(w),
                SigmaProofOfKnowledgeTree::ProveDlog(v) => v.sigma_serialize(w),
            },
            SigmaBoolean::SigmaConjecture(conj) => match conj {
                SigmaConjecture::Cand(c) => c.sigma_serialize(w),
                SigmaConjecture::Cor(c) => c.sigma_serialize(w),
                SigmaConjecture::Cthreshold(c) => c.sigma_serialize(w),
            },
            SigmaBoolean::TrivialProp(_) => Ok(()), // besides opCode no additional bytes
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let op_code = OpCode::sigma_parse(r)?;
        match op_code {
            OpCode::PROVE_DLOG => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(ProveDlog::sigma_parse(r)?),
            )),
            OpCode::PROVE_DIFFIE_HELLMAN_TUPLE => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDhTuple(ProveDhTuple::sigma_parse(r)?),
            )),
            OpCode::ATLEAST => {
                let c = Cthreshold::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(
                    c,
                )))
            }
            OpCode::AND => {
                let c = Cand::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(c)))
            }
            OpCode::OR => {
                let c = Cor::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(c)))
            }
            _ => todo!(),
        }
    }
}

impl SigmaSerializable for ProveDlog {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.h.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let p = EcPoint::sigma_parse(r)?;
        Ok(ProveDlog::new(p))
    }
}

impl SigmaSerializable for ProveDhTuple {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.gv
            .sigma_serialize(w)
            .and(self.hv.sigma_serialize(w))
            .and(self.uv.sigma_serialize(w))
            .and(self.vv.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let gv = EcPoint::sigma_parse(r)?;
        let hv = EcPoint::sigma_parse(r)?;
        let uv = EcPoint::sigma_parse(r)?;
        let vv = EcPoint::sigma_parse(r)?;
        Ok(ProveDhTuple::new(gv, hv, uv, vv))
    }
}
