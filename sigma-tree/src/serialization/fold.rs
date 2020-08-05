use super::op_code::OpCode;
use crate::ast::{CollMethods, Expr};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use sigma_ser::vlq_encode::WriteSigmaVlqExt;

use std::io;

pub struct FoldSerializer {}

impl FoldSerializer {
    pub const OP_CODE: OpCode = OpCode::FOLD;

    pub fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, w: &mut W) -> Result<(), io::Error> {
        match expr {
            Expr::CollM(CollMethods::Fold {
                input,
                zero,
                fold_op,
            }) => {
                input.sigma_serialize(w)?;
                zero.sigma_serialize(w)?;
                fold_op.sigma_serialize(w)?;
                Ok(())
            }
            _ => panic!("expected Fold"),
        }
    }

    pub fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Expr, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let zero = Expr::sigma_parse(r)?;
        let fold_op = Expr::sigma_parse(r)?;
        Ok(Expr::CollM(CollMethods::Fold {
            input: Box::new(input),
            zero: Box::new(zero),
            fold_op: Box::new(fold_op),
        }))
    }
}
