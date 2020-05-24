//! Serializers
use crate::ast::{CollMethods, Constant, Expr};
use fold::FoldSerializer;
use op_code::OpCode;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::ReadSigmaVlqExt,
};
use std::io;

mod constant;
mod data;
mod fold;
pub mod op_code;
mod sigmaboolean;
pub mod types;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: io::Write>(&self, w: &mut W) -> Result<(), io::Error> {
        match self {
            Expr::Const(c) => c.sigma_serialize(w),
            expr => {
                let op_code = self.op_code();
                op_code.sigma_serialize(w)?;
                match expr {
                    Expr::CollM(cm) => match cm {
                        CollMethods::Fold { .. } => FoldSerializer::sigma_serialize(expr, w),
                    },
                    _ => panic!(format!("don't know how to serialize {}", expr)),
                }
            }
        }
    }

    fn sigma_parse<R: io::Read>(r: &mut R) -> Result<Self, SerializationError> {
        let first_byte = r.peek_u8()?;
        if first_byte <= OpCode::LAST_CONSTANT_CODE.value() {
            let constant = Constant::sigma_parse(r)?;
            Ok(Expr::Const(constant))
        } else {
            let op_code = OpCode::sigma_parse(r)?;
            match op_code {
                FoldSerializer::OP_CODE => FoldSerializer::sigma_parse(r),
                o => Err(SerializationError::NotImplementedOpCode(o.value())),
            }
        }
    }
}
