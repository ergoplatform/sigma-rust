use crate::types::SType;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

impl SigmaSerializable for SType {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, _: &mut W) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        todo!()
    }
    fn sigma_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let c = r.get_u8()?;
        if c == 0 {
            Err(SerializationError::InvalidTypePrefix)
        } else {
            todo!();
            // Ok(SType::SAny)
        }
    }
}
