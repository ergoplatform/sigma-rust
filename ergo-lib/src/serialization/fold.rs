use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::ast::coll_methods::CollM;
use crate::ast::expr::Expr;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};

use std::io;

pub struct FoldSerializer {}

impl FoldSerializer {
    pub const OP_CODE: OpCode = OpCode::FOLD;

    pub fn sigma_serialize<W: SigmaByteWrite>(expr: &Expr, w: &mut W) -> Result<(), io::Error> {
        match expr {
            Expr::CollM(CollM::Fold {
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
        Ok(Expr::CollM(CollM::Fold {
            input: Box::new(input),
            zero: Box::new(zero),
            fold_op: Box::new(fold_op),
        }))
    }
}
