use super::SigmaSerializeResult;
use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::has_opcode::{HasOpCode, HasStaticOpCode};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::sigma_protocol::sigma_boolean::ProveDhTuple;
use crate::sigma_protocol::{
    dlog_group::EcPoint,
    sigma_boolean::{ProveDlog, SigmaBoolean, SigmaConjecture, SigmaProofOfKnowledgeTree},
};

use crate::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;

#[allow(clippy::todo)] // until https://github.com/ergoplatform/sigma-rust/issues/338 is implemented
impl SigmaSerializable for SigmaBoolean {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.op_code().sigma_serialize(w)?;
        match self {
            SigmaBoolean::ProofOfKnowledge(proof) => match proof {
                SigmaProofOfKnowledgeTree::ProveDhTuple(v) => v.sigma_serialize(w),
                SigmaProofOfKnowledgeTree::ProveDlog(v) => v.sigma_serialize(w),
            },
            SigmaBoolean::SigmaConjecture(conj) => match conj {
                SigmaConjecture::Cand(_) => todo!(),
                SigmaConjecture::Cor(_) => todo!(),
                SigmaConjecture::Cthreshold(c) => c.sigma_serialize(w),
            },
            SigmaBoolean::TrivialProp(_) => Ok(()), // besides opCode no additional bytes
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let op_code = OpCode::sigma_parse(r)?;
        match op_code {
            ProveDlog::OP_CODE => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(ProveDlog::sigma_parse(r)?),
            )),
            Cthreshold::OP_CODE => {
                let c = Cthreshold::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(
                    c,
                )))
            }
            _ => todo!(),
        }
    }
}

impl SigmaSerializable for ProveDlog {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.h.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let p = EcPoint::sigma_parse(r)?;
        Ok(ProveDlog::new(p))
    }
}

impl SigmaSerializable for ProveDhTuple {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.g.sigma_serialize(w)?;
        self.h.sigma_serialize(w)?;
        self.u.sigma_serialize(w)?;
        self.v.sigma_serialize(w)
    }

    #[allow(clippy::many_single_char_names)]
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let g = EcPoint::sigma_parse(r)?;
        let h = EcPoint::sigma_parse(r)?;
        let u = EcPoint::sigma_parse(r)?;
        let v = EcPoint::sigma_parse(r)?;
        Ok(ProveDhTuple::new(g, h, u, v))
    }
}
