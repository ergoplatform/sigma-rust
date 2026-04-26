use alloc::boxed::Box;

use crate::ergo_tree::ErgoTreeVersion;
use crate::mir::bin_op::BinOp;
use crate::mir::bin_op::BinOpKind;
use crate::mir::bin_op::RelationOp;
use crate::mir::constant::Constant;
use crate::mir::constant::TryExtractInto;
use crate::mir::expr::Expr;
use crate::mir::upcast::Upcast;
use crate::types::stype::SType;

use super::op_code::OpCode;
use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SigmaParsingError;
use super::SigmaSerializable;
use super::SigmaSerializeResult;

pub fn bin_op_sigma_serialize<W: SigmaByteWrite>(
    bin_op: &BinOp,
    w: &mut W,
) -> SigmaSerializeResult {
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
            let arr = [l.try_extract_into::<bool>()?, r.try_extract_into::<bool>()?];
            w.put_bits(&arr)?;
            Ok(())
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
) -> Result<Expr, SigmaParsingError> {
    let tag = r.get_u8()?;
    Ok(if tag == OpCode::COLL_OF_BOOL_CONST.value() {
        let bools = r.get_bits(2)?;
        #[allow(clippy::unwrap_used)]
        BinOp {
            kind: op_kind,
            left: Box::new(Expr::Const((*bools.first().unwrap()).into())),
            right: Box::new(Expr::Const((*bools.get(1).unwrap()).into())),
        }
        .into()
    } else {
        let mut left = Expr::parse_with_tag(r, tag)?;
        let mut right = Expr::sigma_parse(r)?;
        // S58 mirror of TransformingSigmaBuilder.applyUpcast (used by
        // DeserializationSigmaBuilder): for pre-v3 trees, when an arith/comparison op's
        // operands have mismatched numeric types — common after Site 1 strips
        // Upcast(Const, SBigInt) from a ValDef RHS and the use-site ValUse(N) resolves
        // through valDefTypeStore to the inner Const's narrower type — insert Upcast on
        // the smaller operand to restore the original wider arith. Disabled for v3+.
        //
        // S60 narrowing: only fire when at least one operand is a ValUse (the actual
        // production trigger — type came from the pre-populated valDefTypeStore, not
        // from the operand's own structure). Without this gate, arbitrary proptest
        // inputs like `BinOp(Ge, BinOp_SShort, BinOp_SByte)` get a spurious Upcast
        // inserted on parse, breaking ser_roundtrip across ergotree-ir's MIR proptest
        // suite (mir::and / or / if_op / collection / tuple / xor_of / block /
        // coll_filter / coll_forall / apply / bin_op / serialization::expr).
        if r.tree_version() < ErgoTreeVersion::V3
            && is_arith_or_comparison(&op_kind)
            && (matches!(left, Expr::ValUse(_)) || matches!(right, Expr::ValUse(_)))
        {
            let lt = left.tpe();
            let rt = right.tpe();
            if lt != rt && lt.is_numeric() && rt.is_numeric() {
                let widest = numeric_max(&lt, &rt);
                if lt != widest {
                    left = Expr::Upcast(Upcast {
                        input: Box::new(left),
                        tpe: widest.clone(),
                    });
                }
                if rt != widest {
                    right = Expr::Upcast(Upcast {
                        input: Box::new(right),
                        tpe: widest,
                    });
                }
            }
        }
        BinOp {
            kind: op_kind,
            left: Box::new(left),
            right: Box::new(right),
        }
        .into()
    })
}

fn is_arith_or_comparison(kind: &BinOpKind) -> bool {
    matches!(
        kind,
        BinOpKind::Arith(_)
            | BinOpKind::Bit(_)
            | BinOpKind::Relation(
                RelationOp::Ge | RelationOp::Gt | RelationOp::Le | RelationOp::Lt,
            )
    )
}

fn numeric_rank(t: &SType) -> u8 {
    match t {
        SType::SByte => 1,
        SType::SShort => 2,
        SType::SInt => 3,
        SType::SLong => 4,
        SType::SBigInt => 5,
        SType::SUnsignedBigInt => 5,
        _ => 0,
    }
}

fn numeric_max(a: &SType, b: &SType) -> SType {
    if numeric_rank(a) >= numeric_rank(b) {
        a.clone()
    } else {
        b.clone()
    }
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
}
