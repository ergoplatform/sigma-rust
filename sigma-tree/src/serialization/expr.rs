use super::{fold::FoldSerializer, op_code::OpCode};
use crate::ast::{CollMethods, Constant, Expr};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use sigma_ser::vlq_encode;

use std::io;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
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

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let first_byte = match r.peek_u8() {
            Ok(b) => Ok(b),
            Err(_) => {
                let res = r.get_u8(); // get(consume) the error
                assert!(res.is_err());
                res
            }
        }?;
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
