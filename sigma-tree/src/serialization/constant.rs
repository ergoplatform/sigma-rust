use super::data::DataSerializer;
use crate::{ast::Expr, types::SType};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

pub struct ConstantSerializer {}

impl ConstantSerializer {
    pub fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, mut w: W) -> Result<(), io::Error> {
        match expr {
            Expr::Constant { tpe, v } => {
                tpe.sigma_serialize(&mut w)?;
                DataSerializer::sigma_serialize(v, w)
            }
            _ => panic!("constant expected"),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Expr, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let tpe = SType::sigma_parse(&mut r)?;
        let v = DataSerializer::sigma_parse(&tpe, &mut r)?;
        Ok(Expr::Constant { tpe, v })
    }
}
