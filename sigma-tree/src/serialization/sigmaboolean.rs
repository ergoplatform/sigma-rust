use super::op_code::OpCode;
use crate::{data::SigmaBoolean, ecpoint::EcPointType};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;

impl SigmaSerializable for SigmaBoolean {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.op_code().sigma_serialize(w)?;
        match self {
            SigmaBoolean::ProveDHTuple { .. } => todo!(),
            SigmaBoolean::ProveDlog(v) => v.sigma_serialize(w),
            SigmaBoolean::CAND(_) => todo!(),
        }
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let op_code = OpCode::sigma_parse(r)?;
        match op_code {
            OpCode::PROVE_DLOG => {
                let p = EcPointType::sigma_parse(r)?;
                Ok(SigmaBoolean::ProveDlog(p))
            }
            _ => todo!(),
        }
    }
}
