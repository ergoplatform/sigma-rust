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
        // S61 gate: pre-v3 + arith/comparison BinOp where the narrower operand is a
        // shape whose Upcast wrapper the production write→read round-trip may have lost.
        // Two such shapes:
        //
        //   * `ConstPlaceholder` — Site 1 in expr.rs strips `Upcast(Const, _)` on
        //     serialize and emits the constant via segregation, so it comes back as
        //     `ConstPlaceholder` rather than `Upcast(ConstPlaceholder, _)`.
        //   * `ValUse` — Scala's `TransformingSigmaBuilder.applyUpcast` re-wraps
        //     narrower `ValUse` operands whose type came from `valDefTypeStore`. Site 1
        //     does NOT strip `Upcast(ValUse, _)` on serialize (it strips Const-only),
        //     but our compiler also doesn't always emit that wrapper at MIR-lowering
        //     time for cases where Scala does — so the parser-side re-insert covers
        //     both the Scala round-trip AND our own missing-wrapper.
        //
        // Crucially this excludes bare `Expr::Const`. In production every bare `Const`
        // goes through constant segregation and reaches the parser as
        // `ConstPlaceholder`; a bare `Const` operand at parse time is the proptest-only
        // shape (writer constructed without a constant store via
        // `SigmaByteWriter::new(_, None)`). Excluding it keeps
        // `sigma_serialize_roundtrip` identity intact for arbitrary proptest trees
        // while still firing on every production case.
        //
        // Excludes nested operands like `BinOp`, `MethodCall`, etc. — those preserve
        // their Upcast wrapper through serialization unchanged, so re-inserting one
        // would be a fabrication. The proptest counterexamples that the pre-S60 gate
        // (no operand check) failed on all had nested-operand shapes.
        if r.tree_version() < ErgoTreeVersion::V3 && is_arith_or_comparison(&op_kind) {
            let lt = left.tpe();
            let rt = right.tpe();
            if lt != rt && lt.is_numeric() && rt.is_numeric() {
                let widest = numeric_max(&lt, &rt);
                if lt != widest && is_strippable_operand(&left) {
                    left = Expr::Upcast(Upcast {
                        input: Box::new(left),
                        tpe: widest.clone(),
                    });
                }
                if rt != widest && is_strippable_operand(&right) {
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

/// Returns true if `expr` is an operand shape whose Upcast wrapper the
/// production write→read round-trip may have lost — either a `ConstPlaceholder`
/// (Site 1 in expr.rs strips `Upcast(Const, _)` on serialize and emits the
/// constant via segregation, so it comes back as `ConstPlaceholder`) or a
/// `ValUse` (Scala's `TransformingSigmaBuilder.applyUpcast` re-wraps narrower
/// `ValUse`s whose type came from `valDefTypeStore`).
///
/// Crucially this excludes bare `Expr::Const`. In production every bare `Const`
/// goes through constant segregation and reaches the parser as
/// `ConstPlaceholder`, so a bare `Const` operand is the proptest-only shape
/// (the writer was constructed without a constant store). Excluding `Const`
/// here keeps `sigma_serialize_roundtrip` identity intact for arbitrary
/// proptest trees while still firing on every production case.
fn is_strippable_operand(expr: &Expr) -> bool {
    matches!(expr, Expr::ConstPlaceholder(_) | Expr::ValUse(_))
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
