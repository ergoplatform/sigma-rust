use std::io::Error;

use crate::mir::bin_op::BinOp;
use crate::mir::bin_op::BinOpKind;
use crate::mir::constant::Constant;
use crate::mir::constant::TryExtractInto;
use crate::mir::expr::Expr;
use crate::types::stype::SType;

use super::op_code::OpCode;
use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SerializationError;
use super::SigmaSerializable;

pub fn bin_op_sigma_serialize<W: SigmaByteWrite>(bin_op: &BinOp, w: &mut W) -> Result<(), Error> {
    match (*bin_op.clone().left, *bin_op.clone().right) {
        (
            Expr::Const(Constant {
                tpe: SType::SBoolean,
                v: l,
            }),
            Expr::Const(Constant {
                tpe: SType::SBoolean,
                v: r,
            }),
        ) => {
            OpCode::COLL_OF_BOOL_CONST.sigma_serialize(w)?;
            let arr = [
                l.try_extract_into::<bool>().unwrap(),
                r.try_extract_into::<bool>().unwrap(),
            ];
            w.put_bits(&arr)
        }
        _ => {
            bin_op.left.sigma_serialize(w)?;
            bin_op.right.sigma_serialize(w)
        }
    }
}

pub fn bin_op_sigma_parse<R: SigmaByteRead>(
    op_kind: BinOpKind,
    r: &mut R,
) -> Result<Expr, SerializationError> {
    Ok(if r.peek_u8()? == OpCode::COLL_OF_BOOL_CONST.value() {
        let _ = r.get_u8()?; // consume op code byte (peeked above)
        let bools = r.get_bits(2)?;
        BinOp {
            kind: op_kind,
            left: Box::new(Expr::Const(bools.get(0).unwrap().clone().into())),
            right: Box::new(Expr::Const(bools.get(1).unwrap().clone().into())),
        }
        .into()
    } else {
        let left = Expr::sigma_parse(r)?;
        let right = Expr::sigma_parse(r)?;
        BinOp {
            kind: op_kind,
            left: Box::new(left),
            right: Box::new(right),
        }
        .into()
    })
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod proptests {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any_with::<BinOp>(ArbExprParams {tpe: SType::SAny, depth: 0})) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}

#[cfg(test)]
mod tests {
    use test_util::force_any_val_with;

    use super::*;
    use crate::mir::bin_op::RelationOp;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::stype::SType;

    fn test_ser_roundtrip(kind: BinOpKind, left: Expr, right: Expr) {
        let eq_op: Expr = BinOp {
            kind,
            left: Box::new(left),
            right: Box::new(right),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&eq_op), eq_op];
    }

    #[test]
    fn ser_roundtrip_eq() {
        test_ser_roundtrip(
            RelationOp::Eq.into(),
            force_any_val_with::<Expr>(ArbExprParams {
                tpe: SType::SAny,
                depth: 1,
            }),
            force_any_val_with::<Expr>(ArbExprParams {
                tpe: SType::SAny,
                depth: 1,
            }),
        )
    }

    #[test]
    fn ser_roundtrip_neq() {
        test_ser_roundtrip(
            RelationOp::NEq.into(),
            force_any_val_with::<Expr>(ArbExprParams {
                tpe: SType::SAny,
                depth: 1,
            }),
            force_any_val_with::<Expr>(ArbExprParams {
                tpe: SType::SAny,
                depth: 1,
            }),
        )
    }
}
