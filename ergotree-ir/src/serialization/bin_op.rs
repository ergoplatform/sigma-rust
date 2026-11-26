use alloc::boxed::Box;

use crate::mir::bin_op::BinOp;
use crate::mir::bin_op::BinOpKind;
use crate::mir::constant::Constant;
use crate::mir::constant::TryExtractInto;
use crate::mir::expr::Expr;
use crate::types::stype::SType;

use super::op_code::OpCode;
use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SigmaParsingError;
use super::SigmaSerializable;
use super::SigmaSerializeResult;

fn supports_packed_bool_pair(kind: BinOpKind) -> bool {
    matches!(kind, BinOpKind::Relation(_) | BinOpKind::Logical(_))
}

pub fn bin_op_sigma_serialize<W: SigmaByteWrite>(
    bin_op: &BinOp,
    w: &mut W,
) -> SigmaSerializeResult {
    // Only `Relation` (GT/GE/LT/LE/EQ/NEQ) and `Logical` (BinOr/BinAnd/BinXor) use
    // `Relation2Serializer` in reference `sigmastate-interpreter` (`ValueSerializer.scala`);
    // `Arith` and `Bit` use `TwoArgumentsSerializer` and never emit packed bool pairs.
    if supports_packed_bool_pair(bin_op.kind) {
        if let (
            Expr::Const(Constant {
                tpe: SType::SBoolean,
                v: l,
            }),
            Expr::Const(Constant {
                tpe: SType::SBoolean,
                v: r,
            }),
        ) = (&*bin_op.left, &*bin_op.right)
        {
            OpCode::COLL_OF_BOOL_CONST.sigma_serialize(w)?;
            let arr = [
                l.clone().try_extract_into::<bool>()?,
                r.clone().try_extract_into::<bool>()?,
            ];
            w.put_bits(&arr)?;
            return Ok(());
        }
    }
    bin_op.left.sigma_serialize(w)?;
    bin_op.right.sigma_serialize(w)
}

pub fn bin_op_sigma_parse<R: SigmaByteRead>(
    op_kind: BinOpKind,
    r: &mut R,
) -> Result<Expr, SigmaParsingError> {
    // Reference `sigmastate-interpreter` (`ValueSerializer.scala`):
    // - `Relation` and `Logical` use `Relation2Serializer`, which peeks for the packed
    //   boolean pair (`COLL_OF_BOOL_CONST`, 0x85) without a position check, then reads
    //   either the 2-bit pair or two values.
    // - `Arith` and `Bit` use `TwoArgumentsSerializer` and always read two values.
    Ok(
        if supports_packed_bool_pair(op_kind) && r.peek_u8()? == OpCode::COLL_OF_BOOL_CONST.value()
        {
            r.get_u8()?;
            let bools = r.get_bits(2)?;
            #[allow(clippy::unwrap_used)]
            BinOp {
                kind: op_kind,
                left: Box::new(Expr::Const((*bools.first().unwrap()).into())),
                right: Box::new(Expr::Const((*bools.get(1).unwrap()).into())),
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
        },
    )
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
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
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use sigma_test_util::force_any_val_with;

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

    #[test]
    fn packed_bool_pair_restricted_to_relation_and_logical_ops() {
        use crate::mir::bin_op::{ArithOp, BitOp, LogicalOp};

        // Relation and Logical ops use the 3-byte packed bool pair encoding (op, 0x85, bits).
        for (kind, expected_op) in [
            (BinOpKind::Relation(RelationOp::Eq), OpCode::EQ),
            (BinOpKind::Logical(LogicalOp::And), OpCode::BIN_AND),
            (BinOpKind::Logical(LogicalOp::Or), OpCode::BIN_OR),
            (BinOpKind::Logical(LogicalOp::Xor), OpCode::BIN_XOR),
        ] {
            let expr: Expr = BinOp {
                kind,
                left: Box::new(Expr::Const(true.into())),
                right: Box::new(Expr::Const(false.into())),
            }
            .into();
            let bytes = expr.sigma_serialize_bytes().unwrap();
            assert_eq!(
                bytes,
                vec![
                    expected_op.value(),
                    OpCode::COLL_OF_BOOL_CONST.value(),
                    0x01
                ]
            );
            assert_eq!(Expr::sigma_parse_bytes(&bytes).unwrap(), expr);
        }

        // Arith and Bit ops use TwoArgumentsSerializer and must neither parse nor emit 0x85 pairs.
        for (kind, op) in [
            (BinOpKind::Arith(ArithOp::Plus), OpCode::PLUS),
            (BinOpKind::Arith(ArithOp::Minus), OpCode::MINUS),
            (BinOpKind::Arith(ArithOp::Multiply), OpCode::MULTIPLY),
            (BinOpKind::Arith(ArithOp::Divide), OpCode::DIVISION),
            (BinOpKind::Arith(ArithOp::Modulo), OpCode::MODULO),
            (BinOpKind::Arith(ArithOp::Min), OpCode::MIN),
            (BinOpKind::Arith(ArithOp::Max), OpCode::MAX),
            (BinOpKind::Bit(BitOp::BitOr), OpCode::BIT_OR),
            (BinOpKind::Bit(BitOp::BitAnd), OpCode::BIT_AND),
            (BinOpKind::Bit(BitOp::BitXor), OpCode::BIT_XOR),
        ] {
            let wire_packed = [op.value(), OpCode::COLL_OF_BOOL_CONST.value(), 0x03];
            assert!(
                Expr::sigma_parse_bytes(&wire_packed).is_err(),
                "expected parse error for packed bool wire bytes on {:?}",
                kind
            );

            let expr: Expr = BinOp {
                kind,
                left: Box::new(Expr::Const(true.into())),
                right: Box::new(Expr::Const(false.into())),
            }
            .into();
            let bytes = expr.sigma_serialize_bytes().unwrap();
            assert_eq!(bytes, vec![op.value(), 0x01, 0x01, 0x01, 0x00]);
            assert_eq!(Expr::sigma_parse_bytes(&bytes).unwrap(), expr);
        }
    }
}
