//! Serializers
use crate::ast::{CollMethods, Expr};
use constant::ConstantSerializer;
use fold::FoldSerializer;
use op_code::OpCode;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

mod constant;
mod data;
mod fold;
pub mod op_code;
mod types;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        match self {
            Expr::Constant { .. } => ConstantSerializer::sigma_serialize(self, w),
            expr => {
                let op_code = self.op_code();
                op_code.sigma_serialize(&mut w)?;
                ExprSerializers::sigma_serialize(expr, &mut w)
            }
        }
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let first_byte = r.peek_u8()?;
        if first_byte <= OpCode::LAST_CONSTANT_CODE.value() {
            ConstantSerializer::sigma_parse(&mut r)
        } else {
            let op_code = OpCode::sigma_parse(&mut r)?;
            ExprSerializers::sigma_parse(op_code, &mut r)
        }
    }
}

pub struct ExprSerializers {}

impl ExprSerializers {
    pub fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, w: W) -> Result<(), io::Error> {
        match expr {
            Expr::CollM(cm) => match cm {
                CollMethods::Fold { .. } => FoldSerializer::sigma_serialize(expr, w),
            },
            _ => panic!(format!("don't know how to serialize {}", expr)),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(
        op_code: OpCode,
        r: R,
    ) -> Result<Expr, SerializationError> {
        match op_code {
            FoldSerializer::OP_CODE => FoldSerializer::sigma_parse(r),
            o => Err(SerializationError::NotImplementedOpCode(o.value())),
        }
    }
}
