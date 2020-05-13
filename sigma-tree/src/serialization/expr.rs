use crate::ast::Expr;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        match self {
            c @ Constant { .. } => ConstantSerializer::sigma_serialize(self, w),
            expr => {
                let op_code = self.op_code();
                w.put_u8(op_code.0)?;
                ExprSerializers::sigma_serialize(self, w)
            }
        }
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let first_byte = r.peek_u8()?;
        if first_byte <= LAST_CONSTANT_CODE {
            ConstantSerializer::sigma_parse(&mut r)
        } else {
            let op_code = r.get_u8()?;
            ExprSerializers::sigma_parse(&OpCode(op_code), r)
        }
    }
}
