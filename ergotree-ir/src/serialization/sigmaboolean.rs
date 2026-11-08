use super::SigmaSerializeResult;
use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::has_opcode::{HasOpCode, HasStaticOpCode};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::sigma_protocol::sigma_boolean::{
    ProveDhTuple, ProveDlog, SigmaBoolean, SigmaConjecture, SigmaProofOfKnowledgeTree,
};

use ergo_chain_types::EcPoint;

use crate::sigma_protocol::sigma_boolean::cand::Cand;
use crate::sigma_protocol::sigma_boolean::cor::Cor;
use crate::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;

impl SigmaSerializable for SigmaBoolean {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // the 1-byte op code Scala writes for every variant is metered inside
        // `OpCode::sigma_serialize` (PutByteCost)
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

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // Every SigmaBoolean node is one nesting level, released only on success
        // (sigmastate v6.0.6 `SigmaBoolean.serializer.parse`, `SigmaBoolean.scala:71-103`).
        let depth = r.level();
        r.set_level(depth + 1)?;
        let op_code = OpCode::sigma_parse(r)?;
        let sigma_boolean = match op_code {
            ProveDlog::OP_CODE => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(ProveDlog::sigma_parse(r)?),
            )),
            ProveDhTuple::OP_CODE => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDhTuple(ProveDhTuple::sigma_parse(r)?),
            )),
            Cand::OP_CODE => {
                let c = Cand::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(c)))
            }
            Cor::OP_CODE => {
                let c = Cor::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(c)))
            }
            Cthreshold::OP_CODE => {
                let c = Cthreshold::sigma_parse(r)?;
                Ok(SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(
                    c,
                )))
            }
            OpCode::TRIVIAL_PROP_TRUE => Ok(SigmaBoolean::TrivialProp(true)),
            OpCode::TRIVIAL_PROP_FALSE => Ok(SigmaBoolean::TrivialProp(false)),
            _ => Err(SigmaParsingError::Misc(format!(
                "unexpected op code in SigmaBoolean parsing: {:?}",
                op_code
            ))),
        }?;
        r.set_level(r.level().saturating_sub(1))?;
        Ok(sigma_boolean)
    }
}

impl SigmaSerializable for ProveDlog {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.h.sigma_serialize(w)?;
        w.add_put_chunk_cost(EcPoint::GROUP_SIZE);
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let p = EcPoint::sigma_parse(r)?;
        Ok(ProveDlog::new(p))
    }
}

impl SigmaSerializable for ProveDhTuple {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.g.sigma_serialize(w)?;
        w.add_put_chunk_cost(EcPoint::GROUP_SIZE);
        self.h.sigma_serialize(w)?;
        w.add_put_chunk_cost(EcPoint::GROUP_SIZE);
        self.u.sigma_serialize(w)?;
        w.add_put_chunk_cost(EcPoint::GROUP_SIZE);
        self.v.sigma_serialize(w)?;
        w.add_put_chunk_cost(EcPoint::GROUP_SIZE);
        Ok(())
    }

    #[allow(clippy::many_single_char_names)]
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let g = EcPoint::sigma_parse(r)?;
        let h = EcPoint::sigma_parse(r)?;
        let u = EcPoint::sigma_parse(r)?;
        let v = EcPoint::sigma_parse(r)?;
        Ok(ProveDhTuple::new(g, h, u, v))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod depth_limit_tests {
    //! JVM parity: every SigmaBoolean node is one nesting level (sigmastate v6.0.6
    //! `SigmaBoolean.serializer.parse`, `SigmaBoolean.scala:71-103`), up to
    //! `MaxTreeDepth` = 110 per reader.
    use super::*;
    use crate::mir::constant::Constant;
    use alloc::vec::Vec;

    /// A `SigmaProp` constant whose SigmaBoolean is `m` nested CANDs,
    /// `CAND(CAND(… CAND(true, true) …, true), true)`: one data level, then `m`
    /// CAND levels and a leaf level.
    fn cand_chain(m: usize) -> Vec<u8> {
        let and = Cand::OP_CODE.value();
        let t = OpCode::TRIVIAL_PROP_TRUE.value();
        let mut bytes = vec![0x08]; // SSigmaProp type code
        for _ in 0..m {
            bytes.extend([and, 2]);
        }
        bytes.extend([t, t]);
        bytes.extend(vec![t; m - 1]);
        bytes
    }

    #[test]
    fn nested_sigma_booleans_reach_exactly_max_tree_depth() {
        assert!(Constant::sigma_parse_bytes(&cand_chain(108)).is_ok());
        assert!(matches!(
            Constant::sigma_parse_bytes(&cand_chain(109)),
            Err(SigmaParsingError::DeserializeCallDepthExceeded(111))
        ));
    }
}
