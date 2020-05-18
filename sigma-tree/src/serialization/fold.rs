use super::op_code::OpCode;
use crate::ast::{CollMethods, Expr};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

pub struct FoldSerializer {}

impl FoldSerializer {
    pub const OP_CODE: OpCode = OpCode::FOLD;

    pub fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, mut w: W) -> Result<(), io::Error> {
        match expr {
            Expr::CollM(CollMethods::Fold {
                input,
                zero,
                fold_op,
            }) => {
                input.sigma_serialize(&mut w)?;
                zero.sigma_serialize(&mut w)?;
                fold_op.sigma_serialize(&mut w)?;
                Ok(())
            }
            _ => panic!("expected Fold"),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Expr, SerializationError> {
        let input = Expr::sigma_parse(&mut r)?;
        let zero = Expr::sigma_parse(&mut r)?;
        let fold_op = Expr::sigma_parse(&mut r)?;
        Ok(Expr::CollM(CollMethods::Fold {
            input: Box::new(input),
            zero: Box::new(zero),
            fold_op: Box::new(fold_op),
        }))
    }
}
