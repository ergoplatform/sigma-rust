use std::io::Error;

use crate::ast::bin_op::BinOp;
use crate::ast::bin_op::BinOpKind;
use crate::ast::expr::Expr;

use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SerializationError;
use super::SigmaSerializable;

pub fn bin_op_sigma_serialize<W: SigmaByteWrite>(bin_op: &BinOp, w: &mut W) -> Result<(), Error> {
    bin_op.left.sigma_serialize(w)?;
    bin_op.right.sigma_serialize(w)?;
    Ok(())
}

pub fn bin_op_sigma_parse<R: SigmaByteRead>(
    op_kind: BinOpKind,
    r: &mut R,
) -> Result<Expr, SerializationError> {
    let left = Expr::sigma_parse(r)?;
    let right = Expr::sigma_parse(r)?;
    Ok(Box::new(BinOp {
        kind: op_kind,
        left,
        right,
    })
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::bin_op::LogicOp;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;

    fn test_ser_roundtrip(kind: BinOpKind, left: Expr, right: Expr) {
        let eq_op: Expr = Box::new(BinOp { kind, left, right }).into();
        assert_eq![sigma_serialize_roundtrip(&eq_op), eq_op];
    }

    #[test]
    fn ser_roundtrip_eq() {
        test_ser_roundtrip(
            LogicOp::Eq.into(),
            force_any_val::<Expr>(),
            force_any_val::<Expr>(),
        )
    }

    #[test]
    fn ser_roundtrip_neq() {
        test_ser_roundtrip(
            LogicOp::NEq.into(),
            force_any_val::<Expr>(),
            force_any_val::<Expr>(),
        )
    }
}
