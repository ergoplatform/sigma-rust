use super::data::DataSerializer;
use crate::{ast::Constant, types::SType};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

impl SigmaSerializable for Constant {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.tpe.sigma_serialize(w)?;
        DataSerializer::sigma_serialize(&self.v, w)
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let tpe = SType::sigma_parse(r)?;
        let v = DataSerializer::sigma_parse(&tpe, r)?;
        Ok(Constant { tpe, v })
    }
}
