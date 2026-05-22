use ergotree_ir::mir::bin_op::ArithOp;
use ergotree_ir::mir::bin_op::BinOp;
use ergotree_ir::mir::bin_op::BinOpKind;
use ergotree_ir::mir::bin_op::BitOp;
use ergotree_ir::mir::bin_op::LogicalOp;
use ergotree_ir::mir::bin_op::RelationOp;
use ergotree_ir::mir::bit_inversion::BitInversion;
use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt;
use ergotree_ir::mir::byte_array_to_long::ByteArrayToLong;
use ergotree_ir::mir::calc_blake2b256::CalcBlake2b256;
use ergotree_ir::mir::calc_sha256::CalcSha256;
use ergotree_ir::mir::coll_append::Append;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::coll_exists::Exists;
use ergotree_ir::mir::coll_filter::Filter;
use ergotree_ir::mir::coll_fold::Fold;
use ergotree_ir::mir::coll_forall::ForAll;
use ergotree_ir::mir::coll_map::Map;
use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::coll_slice::Slice;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::downcast::Downcast;
use ergotree_ir::mir::exponentiate::Exponentiate;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::extract_amount::ExtractAmount;
use ergotree_ir::mir::extract_bytes::ExtractBytes;
use ergotree_ir::mir::extract_creation_info::ExtractCreationInfo;
use ergotree_ir::mir::extract_id::ExtractId;
use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::func_value::FuncArg;
use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::get_var::GetVar;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::if_op::If;
use ergotree_ir::mir::logical_not::LogicalNot;
use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
use ergotree_ir::mir::method_call::MethodCall;
use ergotree_ir::mir::negation::Negation;
use ergotree_ir::mir::option_get::OptionGet;
use ergotree_ir::mir::option_is_defined::OptionIsDefined;
use ergotree_ir::mir::property_call::PropertyCall;
use ergotree_ir::mir::select_field::SelectField;
use ergotree_ir::mir::select_field::TupleFieldIndex;
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::mir::sigma_or::SigmaOr;
use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
use ergotree_ir::mir::sigma_prop_is_proven::SigmaPropIsProven;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::tree_lookup::TreeLookup;
use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
use ergotree_ir::mir::upcast::Upcast;
use ergotree_ir::mir::val_def::ValDef;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::mir::xor::Xor;
use ergotree_ir::mir::xor_of::XorOf;
use ergotree_ir::source_span::Spanned;
use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType;
use hir::BinaryOp;
use rowan::TextRange;
use std::collections::HashMap;

use crate::error::pretty_error_desc;
use crate::hir;

#[derive(Debug, PartialEq, Eq)]
pub struct MirLoweringError {
    msg: String,
    span: TextRange,
}

impl MirLoweringError {
    pub fn new(msg: String, span: TextRange) -> Self {
        Self { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

/// Numeric type promotion rank: higher number = wider type.
/// Returns None for non-numeric types.
fn numeric_rank(tpe: &SType) -> Option<u8> {
    match tpe {
        SType::SByte => Some(1),
        SType::SShort => Some(2),
        SType::SInt => Some(3),
        SType::SLong => Some(4),
        SType::SBigInt => Some(5),
        _ => None,
    }
}

/// When mixing numeric types in a BinOp, insert Upcast on the narrower operand
/// to promote it to the wider type. This matches the Scala ErgoScript compiler's
/// implicit numeric promotion (e.g., Int * Long → Upcast(Int, SLong) * Long,
/// Long * BigInt → Upcast(Long, SBigInt) * BigInt).
/// `ByIndex` requires an `SInt` index expression. If the source has a narrower
/// integer type (e.g. `SByte` or `SShort` from `getVar[Byte]` arithmetic), Sigma's
/// Scala compiler implicitly upcasts; mirror that here.
fn upcast_index_to_int(idx: Expr) -> Expr {
    if idx.tpe() == SType::SInt {
        idx
    } else if matches!(idx.tpe(), SType::SByte | SType::SShort) {
        numeric_upcast(idx, SType::SInt)
    } else {
        idx
    }
}

fn numeric_upcast_pair(l: Expr, r: Expr) -> (Expr, Expr) {
    let lt = l.tpe();
    let rt = r.tpe();
    if lt == rt {
        return (l, r);
    }
    let lr = numeric_rank(&lt);
    let rr = numeric_rank(&rt);
    match (lr, rr) {
        (Some(l_rank), Some(r_rank)) if l_rank < r_rank => (numeric_upcast(l, rt), r),
        (Some(l_rank), Some(r_rank)) if l_rank > r_rank => (l, numeric_upcast(r, lt)),
        _ => (l, r),
    }
}

/// Insert Upcast to promote a narrower numeric type to a wider one.
///
/// We deliberately do NOT fold `Upcast(Const(N: SInt), SBigInt)` to
/// `Const(BigInt256(N))`. Scala's TransformingSigmaBuilder keeps the
/// `Upcast(intConst, BigInt)` wrapper at use sites; serialization Site 1
/// (`expr_serializer::write_expr`) strips the wrapper for pre-v3 trees so the
/// constant lands in the pool with its source-level `SInt` type, and parser
/// Site 2 (`bin_op::parse_bin_op`) re-inserts the wrapper at use sites when
/// operand types differ. Folding it here drops the wrapper before serialization
/// so the constant is segregated as `SBigInt` instead of `SInt`, which makes
/// our pool encoding diverge from NODE on every fixture mixing bare int
/// literals with BigInt arithmetic (e.g. spectrum N2T/T2T pool's `FeeDenom`).
///
/// The implicit `Upcast(Const(N: SInt), SLong)` fold is performed at HIR widen
/// time (`widen_numeric_literals` / `widen_binop_literal_pair`), not here.
/// Reason: by MIR-time we cannot distinguish a `Const(SInt)` that came from a
/// source-level inline literal (Scala folds — sigmausd `fee * -1`) from one
/// that came from a val-bound substitution (Scala does NOT fold — oracle
/// `val maxDev = 5; lastData * maxDev`, duckpools `val updateFrequency = 120;
/// deltaHeight >= updateFrequency`). Empirical probe `probe_int_to_long_upcast_fold`
/// confirms: literal Int operands (positive or negative) at a wider-Long BinOp
/// position fold to Const(SLong) in NODE; val-substituted Int operands stay as
/// Const(SInt) with a separate Upcast wrapper. HIR widen runs before
/// `constant_fold` substitutes ValUse → Literal and so preserves the
/// distinction; the fold has been moved there to match the asymmetry.
fn numeric_upcast(expr: Expr, target: SType) -> Expr {
    Upcast::new(expr, target).expect("numeric upcast").into()
}

/// Look up a V6 numeric SMethod by name on the given numeric receiver type.
/// Each per-type METHODS Vec already contains methods specialized for that
/// receiver (the `STypeVar::t()` substitution is done at lazy_static time),
/// so the returned SMethod can be passed straight into MethodCall::new /
/// PropertyCall::new without further specialize_for.
fn lookup_numeric_method(
    receiver: &SType,
    name: &str,
) -> Option<ergotree_ir::types::smethod::SMethod> {
    use ergotree_ir::types::snumeric;
    let methods: &[ergotree_ir::types::smethod::SMethod] = match receiver {
        SType::SByte => &snumeric::sbyte::METHODS,
        SType::SShort => &snumeric::sshort::METHODS,
        SType::SInt => &snumeric::sint::METHODS,
        SType::SLong => &snumeric::slong::METHODS,
        SType::SBigInt => &snumeric::sbigint::METHODS,
        SType::SUnsignedBigInt => &snumeric::sunsignedbigint::METHODS,
        _ => return None,
    };
    methods.iter().find(|m| m.name() == name).cloned()
}

/// Constant-fold a numeric Downcast on a literal: `Const(intLit).toSmaller`
/// becomes a `Const` of the smaller type when the value fits the target range.
/// Mirrors Scala's graph-IR behavior, where `Downcast(Const, T)` folds to
/// `Const_T` at compile time (see `sigma.compiler.ir.TreeBuilding` /
/// `GraphBuilding`). Out-of-range values fall through to runtime Downcast,
/// which preserves the existing Rust behavior.
///
/// Only applied to narrowing casts. Upcasts on literals are left as runtime
/// `Upcast` nodes to match Scala (the only Upcast fold Scala performs is for
/// `toBigInt`, handled separately at the toBigInt arm).
fn fold_numeric_downcast_on_const(obj: &Expr, target: SType) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    use num_traits::ToPrimitive;
    let c = match obj {
        Expr::Const(c) => c,
        _ => return None,
    };
    let as_i64: i64 = match &c.v {
        Literal::Byte(v) => *v as i64,
        Literal::Short(v) => *v as i64,
        Literal::Int(v) => *v as i64,
        Literal::Long(v) => *v,
        Literal::BigInt(bi) => bi.to_i64()?,
        _ => return None,
    };
    match target {
        SType::SByte => i8::try_from(as_i64).ok().map(|v| Constant::from(v).into()),
        SType::SShort => i16::try_from(as_i64).ok().map(|v| Constant::from(v).into()),
        SType::SInt => i32::try_from(as_i64).ok().map(|v| Constant::from(v).into()),
        SType::SLong => Some(Constant::from(as_i64).into()),
        _ => None,
    }
}

/// Fold an explicit `.toBigInt` on an Int `Const` to a `Const(SBigInt)`.
/// Mirrors Scala's graph-IR behavior on `Upcast(Const(SInt), SBigInt)` for the
/// **explicit `.toBigInt`** call path (Select-method lowering).
///
/// Only Int literals are folded — `LongLit.toBigInt` is left as
/// `Upcast(Const(SLong), SBigInt)` so Site 1 can strip it and leave SLong in
/// the pool (cf. DuckPools `InterestDenomination = 100000000L`, where the HIR
/// constant_fold pass inlines the literal Long into the `.toBigInt` receiver
/// and Scala still encodes the pool slot as SLong, not SBigInt).
///
/// The implicit `numeric_upcast` BinOp-coercion path is intentionally NOT
/// folded either — see the `numeric_upcast` doc comment for the spectrum
/// FeeDenom rationale (parser re-inserts wrapper at use sites, pool stays SInt).
fn fold_to_bigint_on_const(obj: &Expr) -> Option<Expr> {
    use ergotree_ir::bigint256::BigInt256;
    use ergotree_ir::mir::constant::Literal;
    let c = match obj {
        Expr::Const(c) => c,
        _ => return None,
    };
    let bi: BigInt256 = match &c.v {
        Literal::Int(v) => BigInt256::from(*v as i64),
        _ => return None,
    };
    Some(Constant::from(bi).into())
}

/// Fold an explicit `.toLong` on an Int `Const` to a `Const(SLong)`.
///
/// Mirrors Scala's graph-IR behavior: `NumericToLong[Int](Const(SInt N))` falls
/// through `rewriteUnOp` (DefRewriting.scala:92) to the default `propagateUnOp`
/// arm (DefRewriting.scala:173-185), which folds any Const arg via
/// `op.applySeq(xVal)`. Empirically verified at the Ergo node:
/// `{ val l = 18207.toLong; sigmaProp(l >= 0L) }` compiles to the
/// `sigmaProp(true)` shape `10010101d17300` — only possible if Scala folded the
/// `IntLit.toLong` to `Const(SLong)` so that the downstream `>=` Const-Const
/// fold (628ddcab) and `BoolToSigmaProp(Const)` (left unfolded per
/// `project_bool_to_sigmaprop_not_folded.md`) produce the final tree.
///
/// **Scope is narrow on purpose** — only `Int → Long` widening folds. Other
/// widening arms (`Byte → Short/Int/Long`, `Short → Int/Long`) empirically do
/// NOT fold in Scala (verified by p2sAddress probes on `((25).toByte).toLong`,
/// `((25).toShort).toLong`, `((25).toShort).toInt`, `((25).toByte).toInt` — all
/// produced distinct, non-`sigmaProp(true)` addresses). Likely Scala's
/// parser/typer recognizes `intLit.toLong` as a numeric literal extension
/// directly (analogous to writing `18207L`), while smaller-source Upcast on
/// Const stays as a runtime `Upcast` node.
///
/// Implicit `numeric_upcast` BinOp-coercion path is unaffected (Spectrum's
/// `val FeeDenom = 1000` use sites still flow through that helper, not this
/// arm).
fn fold_int_lit_to_long(obj: &Expr) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    let c = match obj {
        Expr::Const(c) => c,
        _ => return None,
    };
    match &c.v {
        Literal::Int(v) => Some(Constant::from(*v as i64).into()),
        _ => None,
    }
}

fn is_numeric_tpe(tpe: &Option<SType>) -> bool {
    matches!(
        tpe,
        Some(
            SType::SByte
                | SType::SShort
                | SType::SInt
                | SType::SLong
                | SType::SBigInt
                | SType::SUnsignedBigInt
        )
    )
}

/// Mirror Scala graph-IR fold: `OrderingLT/LTEQ/GT/GTEQ(Const, Const) → Const(Boolean)`.
///
/// Scala's `rewriteBinOp` falls through to the default `case _ => propagateBinOp`
/// arm (DefRewriting.scala:166) for the four Ordering BinOps, which folds any
/// two-Const application via `op.applySeq(xVal, yVal)`. Without this Rust-side
/// mirror, Rust emits an unfolded `BinOp(Lt|Le|Gt|Ge, Const, Const)` while Scala
/// emits a single `Const(true|false)` — diverging the encoded ErgoTree bytes.
///
/// EQ/NEQ are intentionally excluded — see `project_eq_neq_not_fold_sibling.md`:
/// Scala's Equals/NotEquals arm does NOT delegate to `propagateBinOp` and only
/// folds via Ref equality (which catches hash-consed equal-value Consts) and a
/// Boolean-specialization sub-rule. For differing-value non-Bool Consts, Scala
/// emits the unfolded EQ/NEQ node; mirroring requires a more nuanced fold than
/// the Ordering ops, deferred to a separate change.
///
/// Conservative scope: only fold when both args are `Const` of the same numeric
/// `Literal` variant. Mixed Const+ValUse / ValUse+ValUse take the unchanged
/// runtime path. Type-mismatched Const pairs (e.g. Byte vs Long after some
/// upstream divergence) also fall through.
///
/// **BigInt / UnsignedBigInt deliberately excluded** — empirical p2sAddress probe
/// (treeVersion=0): differing-value `(5).toBigInt op (10).toBigInt` for all four
/// Ordering ops emits 4 distinct unfolded addresses, none matching the
/// `sigmaProp(true|false)` baselines. Scala leaves BigInt Const+Const Ordering
/// as a runtime BinOp — same per-arm asymmetry as the arithmetic side
/// (`project_bigint_arith_not_folded.md`). See `project_reflexive_ordering_not_folded.md`
/// for the comparison-side probe table; folding the BigInt arms here regresses
/// `numeric_cmp_095/096/097/098` (Rust 7 bytes folded vs Scala 18 bytes unfolded).
fn fold_compare_const_const(op: &BinaryOp, l: &Expr, r: &Expr) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    use std::cmp::Ordering;
    if !matches!(
        op,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
    ) {
        return None;
    }
    let (lc, rc) = match (l, r) {
        (Expr::Const(lc), Expr::Const(rc)) => (lc, rc),
        _ => return None,
    };
    let ord = match (&lc.v, &rc.v) {
        (Literal::Byte(a), Literal::Byte(b)) => a.cmp(b),
        (Literal::Short(a), Literal::Short(b)) => a.cmp(b),
        (Literal::Int(a), Literal::Int(b)) => a.cmp(b),
        (Literal::Long(a), Literal::Long(b)) => a.cmp(b),
        _ => return None,
    };
    let result = match op {
        BinaryOp::Lt => ord == Ordering::Less,
        BinaryOp::Le => ord != Ordering::Greater,
        BinaryOp::Gt => ord == Ordering::Greater,
        BinaryOp::Ge => ord != Ordering::Less,
        _ => unreachable!(),
    };
    Some(Constant::from(result).into())
}

/// Detect Byte/Short arithmetic overflow on Const+Const operands. Scala rejects
/// these programs at compile time with "Byte overflow" / "Short overflow" via
/// `propagateBinOp`'s `op.applySeq(a, b)` raising — empirically observed by
/// p2sAddress probe and recorded in `project_bigint_arith_not_folded.md`. Returns
/// the type-name string when the Plus/Minus/Multiply operation would overflow;
/// the lower-site converts that into a `MirLoweringError` so Rust mirrors Scala's
/// reject behavior instead of silently emitting the unfolded BinOp. Divide/Modulo
/// stay as runtime BinOps in Scala (probed) so we leave them unchanged here.
fn check_byte_short_overflow(op: &BinaryOp, l: &Expr, r: &Expr) -> Option<&'static str> {
    use ergotree_ir::mir::constant::Literal;
    if !matches!(
        op,
        BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Multiply
    ) {
        return None;
    }
    let (lc, rc) = match (l, r) {
        (Expr::Const(lc), Expr::Const(rc)) => (lc, rc),
        _ => return None,
    };
    match (&lc.v, &rc.v) {
        (Literal::Byte(a), Literal::Byte(b)) => {
            let ok = match op {
                BinaryOp::Plus => a.checked_add(*b).is_some(),
                BinaryOp::Minus => a.checked_sub(*b).is_some(),
                BinaryOp::Multiply => a.checked_mul(*b).is_some(),
                _ => true,
            };
            if !ok {
                Some("Byte")
            } else {
                None
            }
        }
        (Literal::Short(a), Literal::Short(b)) => {
            let ok = match op {
                BinaryOp::Plus => a.checked_add(*b).is_some(),
                BinaryOp::Minus => a.checked_sub(*b).is_some(),
                BinaryOp::Multiply => a.checked_mul(*b).is_some(),
                _ => true,
            };
            if !ok {
                Some("Short")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Mirror Scala graph-IR fold: `Plus/Minus/Multiply/Divide/Modulo(Const, Const) → Const`
/// for `SByte` / `SShort` / `SInt` / `SLong`.
///
/// Same `propagateBinOp` default arm at `DefRewriting.scala:166` that powers the
/// Ordering fold (628ddcab) — none of the five arithmetic ArithOps have a custom
/// `rewriteBinOp` arm. Empirical p2sAddress probe across the 25 (op × type)
/// combinations confirmed: Byte/Short/Int/Long all fold for all five ops; BigInt
/// does NOT fold (distinct addresses across all five BigInt arms — captured in
/// `project_bigint_arith_not_folded.md`).
///
/// Per-arm corner cases:
///   - **Overflow.** Scala's `propagateBinOp` invokes `op.applySeq(a, b)` which
///     may raise (Byte/Short overflow at compile time was empirically observed —
///     `(100.toByte) + (100.toByte)` is rejected by p2sAddress with "Byte
///     overflow"). Only fold when the checked operation is in-range; otherwise
///     leave the unfolded BinOp so Rust mirrors Scala's runtime-evaluation arm
///     (or the program is rejected upstream by another pass; either way, no
///     bytes-divergence vs Scala for the no-overflow programs that compile).
///   - **Divide / Modulo by zero.** Scala leaves these unfolded (probed:
///     `{ val v = 10 / 0; sigmaProp(v >= 0) }` produces a distinct address from
///     `sigmaProp(true)`, i.e. Scala emits the runtime division so it errors at
///     evaluation, not compile time). Mirror: skip the fold when divisor is
///     zero.
///
/// **BigInt deliberately excluded** — Scala's distinct addresses for all five
/// BigInt arms (`{ (5).toBigInt + (10).toBigInt; ... }` etc.) prove it does not
/// fold. Add it to Rust and we regress every BigInt-arith program — the
/// xorOf-style trap from `project_xorof_not_fold_sibling.md`.
///
/// Conservative scope: only fold when both args are `Const` of the same numeric
/// `Literal` variant (Byte+Byte, Short+Short, Int+Int, Long+Long). Mixed-type
/// Const pairs already get `numeric_upcast_pair`'d upstream — for the four
/// widening arms whose `Upcast(Const, _)` Rust does NOT fold (Byte→Short/Int/Long,
/// Short→Int/Long; see `project_numeric_upcast_const_per_arm_asymmetry.md`),
/// one operand stays as `Upcast(...)` here and falls through to the unchanged
/// runtime BinOp path, matching Scala.
fn fold_arith_const_const(op: &BinaryOp, l: &Expr, r: &Expr) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    if !matches!(
        op,
        BinaryOp::Plus
            | BinaryOp::Minus
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
    ) {
        return None;
    }
    let (lc, rc) = match (l, r) {
        (Expr::Const(lc), Expr::Const(rc)) => (lc, rc),
        _ => return None,
    };
    let result: Constant = match (&lc.v, &rc.v) {
        (Literal::Byte(a), Literal::Byte(b)) => {
            let v = match op {
                BinaryOp::Plus => a.checked_add(*b)?,
                BinaryOp::Minus => a.checked_sub(*b)?,
                BinaryOp::Multiply => a.checked_mul(*b)?,
                BinaryOp::Divide => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_div(*b)?
                }
                BinaryOp::Modulo => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_rem(*b)?
                }
                _ => unreachable!(),
            };
            v.into()
        }
        (Literal::Short(a), Literal::Short(b)) => {
            let v = match op {
                BinaryOp::Plus => a.checked_add(*b)?,
                BinaryOp::Minus => a.checked_sub(*b)?,
                BinaryOp::Multiply => a.checked_mul(*b)?,
                BinaryOp::Divide => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_div(*b)?
                }
                BinaryOp::Modulo => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_rem(*b)?
                }
                _ => unreachable!(),
            };
            v.into()
        }
        (Literal::Int(a), Literal::Int(b)) => {
            let v = match op {
                BinaryOp::Plus => a.checked_add(*b)?,
                BinaryOp::Minus => a.checked_sub(*b)?,
                BinaryOp::Multiply => a.checked_mul(*b)?,
                BinaryOp::Divide => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_div(*b)?
                }
                BinaryOp::Modulo => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_rem(*b)?
                }
                _ => unreachable!(),
            };
            v.into()
        }
        (Literal::Long(a), Literal::Long(b)) => {
            let v = match op {
                BinaryOp::Plus => a.checked_add(*b)?,
                BinaryOp::Minus => a.checked_sub(*b)?,
                BinaryOp::Multiply => a.checked_mul(*b)?,
                BinaryOp::Divide => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_div(*b)?
                }
                BinaryOp::Modulo => {
                    if *b == 0 {
                        return None;
                    }
                    a.checked_rem(*b)?
                }
                _ => unreachable!(),
            };
            v.into()
        }
        // BigInt / UnsignedBigInt: deliberately NOT folded — Scala leaves the
        // unfolded BinOp at the graph IR level (verified empirically via
        // p2sAddress on all five op × BigInt arms; see commit message body).
        _ => return None,
    };
    Some(result.into())
}

/// Mirror Scala graph-IR fold: `OrderingMin/Max(Const, Const) → Const` for
/// `SByte` / `SShort` / `SInt` / `SLong`.
///
/// Sibling extension of `8b1843d5` (Long-only) — the same `propagateBinOp`
/// default arm at `DefRewriting.scala:166` powers `OrderingMin/Max` for every
/// numeric type. Per-arm asymmetry probe via p2sAddress on the local node:
/// Byte/Short/Int/Long all collapse `min/max(Const, Const) <= Const` programs
/// to `sigmaProp(true)` (= baseline `4MQyML64GnzMxZgm`); BigInt does NOT fold
/// (distinct addresses, same shape as `project_bigint_arith_not_folded.md`'s
/// arithmetic asymmetry — `OrderingMin/Max[BigInt]` falls into the same
/// non-folding bucket).
///
/// Conservative scope: only fold same-Literal-variant pairs (mirrors
/// `c46577cd`'s arithmetic-fold scope). Mixed-type Const pairs from implicit
/// `numeric_upcast` are unreachable for `min`/`max` because the parser-level
/// callee resolution forces both args to a single numeric type, but the
/// guard is left in place defensively.
fn fold_min_max_const_const(is_min: bool, l: &Expr, r: &Expr) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    let (lc, rc) = match (l, r) {
        (Expr::Const(lc), Expr::Const(rc)) => (lc, rc),
        _ => return None,
    };
    let result: Constant = match (&lc.v, &rc.v) {
        (Literal::Byte(a), Literal::Byte(b)) => {
            if is_min { std::cmp::min(*a, *b) } else { std::cmp::max(*a, *b) }.into()
        }
        (Literal::Short(a), Literal::Short(b)) => {
            if is_min { std::cmp::min(*a, *b) } else { std::cmp::max(*a, *b) }.into()
        }
        (Literal::Int(a), Literal::Int(b)) => {
            if is_min { std::cmp::min(*a, *b) } else { std::cmp::max(*a, *b) }.into()
        }
        (Literal::Long(a), Literal::Long(b)) => {
            if is_min { std::cmp::min(*a, *b) } else { std::cmp::max(*a, *b) }.into()
        }
        // BigInt deliberately NOT folded — verified empirically.
        _ => return None,
    };
    Some(result.into())
}

/// Mirror Scala's `rewriteBinOp` Equals/NotEquals arm
/// (DefRewriting.scala:99-128): `if (x == y) Const(true|false)` — Ref-equality
/// after Scalan hash-cons.
///
/// Two cases lower to the same Scala-side fold; this Rust mirror catches both
/// at MIR-stage (HIR-stage already handles Long/Int Const+Const same-value
/// directly via `hir/optimize.rs`, but several MIR-stage rewrites — Coll.size,
/// IntLit.toLong, arith Const+Const — only materialize Consts here, so a HIR
/// fold cannot see them):
///
///   (1) **Same-value `Const+Const`** of any equality-deciding `Literal`
///       variant (Byte/Short/Int/Long/BigInt/UnsignedBigInt/ByteArray/
///       GroupElement/etc.). Empirically all probed types fold same-value EQ
///       at the Scala graph IR — the `if (x == y)` Ref-equality arm fires
///       because two `Const(v)` with structurally equal `v` hash-cons to the
///       same `Ref`. Differing-value Const+Const for non-Bool types stays
///       unfolded (see `project_eq_neq_not_fold_sibling.md`).
///
///   (2) **Same-Expr structural equality** — covers `byteArrayToBigInt(SELF.id)
///       == byteArrayToBigInt(SELF.id)` (walker_001), `HEIGHT == HEIGHT`,
///       `SELF.id == SELF.id`, repeated `ValUse` of the same id. Empirically
///       confirmed to fold to `sigmaProp(true)` across all of these shapes.
///
///   (3) **Bool specialization** — `EQ(x_Bool, Const(b))` → `x_Bool` (b=true)
///       or `Not(x_Bool)` (b=false); NEQ mirrors. Symmetric in operand order
///       (Scala checks RHS-Const first, then LHS-Const). The non-Const side
///       must be Bool-typed; gated on exactly-one-side-Const so Const+Const
///       falls to (1). Mirror of `DefRewriting.scala:99-128` Equals/NotEquals
///       arm. Empirically confirmed against 8 variants (4 op×side combinations
///       × 2 directions) — all match the `x_Bool`/`Not(x_Bool)` baselines
///       exactly. Tautology fold (`x || !x → true`) NOT applied by Scala —
///       single-step rewrite only, no cascade.
///
/// **Scope: ONLY EQ/NEQ.** Per `project_reflexive_ordering_not_folded.md`,
/// Scala has no Ref-equality fold for `OrderingLT/LTEQ/GT/GTEQ` —
/// `HEIGHT >= HEIGHT` and `bi >= bi` stay as runtime BinOps. Do NOT extend
/// the same-Expr fold to `Lt/Le/Gt/Ge`.
fn fold_eq_neq(op: &BinaryOp, l: &Expr, r: &Expr) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    let op_is_eq = match op {
        BinaryOp::Eq => true,
        BinaryOp::Neq => false,
        _ => return None,
    };
    if let (Expr::Const(lc), Expr::Const(rc)) = (l, r) {
        if lc.v == rc.v {
            return Some(Constant::from(op_is_eq).into());
        }
        // Differing-value Const+Const for non-Bool: leave unfolded.
        // (Bool Const+Const arrives here already collapsed by HIR-stage.)
        return None;
    }
    if l == r {
        return Some(Constant::from(op_is_eq).into());
    }
    // Bool specialization: exactly one side is Const(Boolean), the other side
    // is a Bool-typed non-Const Expr. Mirrors DefRewriting.scala's Equals
    // arm (RHS-Const checked first, then LHS-Const).
    let bool_const_other: Option<(bool, &Expr)> = match (l, r) {
        (other, Expr::Const(c)) if matches!(c.v, Literal::Boolean(_)) => {
            if let Literal::Boolean(b) = c.v {
                (other.tpe() == SType::SBoolean).then_some((b, other))
            } else {
                None
            }
        }
        (Expr::Const(c), other) if matches!(c.v, Literal::Boolean(_)) => {
            if let Literal::Boolean(b) = c.v {
                (other.tpe() == SType::SBoolean).then_some((b, other))
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some((b, other)) = bool_const_other {
        let folded = if op_is_eq == b {
            other.clone()
        } else {
            LogicalNot::try_build(other.clone())
                .ok()?
                .into()
        };
        return Some(folded);
    }
    None
}

/// Mirror Scala graph-IR fold: `Coll.length` on a known-length receiver folds
/// to `Const(Int(n))` at build time.
///
/// `IRContext.rewriteDef` (sc/.../IRContext.scala:105-119) handles two cases:
///   - `CollConst(coll, _).length → coll.length`
///   - `CBM.fromItems(_, items, _).length → items.length`
///
/// Both fold paths produce a `Const(Int)` regardless of element types or
/// whether elements are themselves Const. The mkMethodCall flags on
/// `CollConstMethods.length` (CollsImpl.scala:45-50) are
/// `isAdapterCall=true, neverInvoke=false` and the body has no version-context
/// branch, so tryInvoke would also fold via reflection — but the rewriteDef
/// arm catches it earlier.
///
/// Without this Rust mirror, `Coll[Long](157734L).size` lowers to
/// `SizeOf{ input: Collection::Exprs{...} }` while Scala emits `Const(1)`,
/// diverging the tree bytes.
///
/// Conservative scope: only fold when receiver is one of
///   - `Expr::Const(Constant{ v: Literal::Coll(_), .. })` — CollConst case.
///   - `Expr::Collection(Collection::Exprs{ items, .. })` — `Coll(...)` form.
///   - `Expr::Collection(Collection::BoolConstants(bools))` — bool literal form.
///
/// Other receivers (`INPUTS`, `box.tokens`, `box.R4.get.coll`, ExtractRegisterAs,
/// Map/Filter results, etc.) take the unchanged `SizeOf` path.
fn fold_coll_size_on_known_length(obj: &Expr) -> Option<Expr> {
    use ergotree_ir::mir::constant::Literal;
    let n: usize = match obj {
        Expr::Const(Constant {
            v: Literal::Coll(coll_kind),
            ..
        }) => coll_kind.len(),
        Expr::Collection(Collection::Exprs { items, .. }) => items.len(),
        Expr::Collection(Collection::BoolConstants(bools)) => bools.len(),
        _ => return None,
    };
    Some(Constant::from(n as i32).into())
}

pub fn lower(hir_expr: hir::Expr) -> Result<Expr, MirLoweringError> {
    let mir: Expr = match &hir_expr.kind {
        hir::ExprKind::GlobalVars(hir) => match hir {
            hir::GlobalVars::Height => GlobalVars::Height.into(),
            hir::GlobalVars::SelfBox => GlobalVars::SelfBox.into(),
            hir::GlobalVars::Inputs => GlobalVars::Inputs.into(),
            hir::GlobalVars::Outputs => GlobalVars::Outputs.into(),
            hir::GlobalVars::GroupGenerator => {
                // NODE v6.1.x lowers `groupGenerator` as `Global.groupGenerator`
                // PropertyCall, not the standalone `GlobalVars::GroupGenerator`
                // opcode. Match that to keep byte parity (saves -3B per use).
                PropertyCall::new(
                    Expr::Global,
                    ergotree_ir::types::sglobal::GROUP_GENERATOR_METHOD.clone(),
                )
                .map(|pc| {
                    Expr::PropertyCall(Spanned {
                        source_span: ergotree_ir::source_span::SourceSpan::empty(),
                        expr: pc,
                    })
                })
                .map_err(|e| {
                    MirLoweringError::new(format!("groupGenerator lower: {e:?}"), hir_expr.span)
                })?
            }
            hir::GlobalVars::MinerPubKey => GlobalVars::MinerPubKey.into(),
        },
        hir::ExprKind::Ident(_) => {
            return Err(MirLoweringError::new(
                format!("MIR error: Unresolved Ident {0:?}", hir_expr),
                hir_expr.span,
            ))
        }
        hir::ExprKind::Binary(hir) => {
            let l = lower(*hir.lhs.clone())?;
            let r = lower(*hir.rhs.clone())?;
            // Coll concatenation `xs ++ ys` → Append(xs, ys)
            if matches!(hir.op.node, BinaryOp::ConcatColl) {
                return Ok(Append::new(l, r)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into());
            }
            // SigmaProp-level && / || → SigmaAnd / SigmaOr
            // If either side is SigmaProp, auto-promote the Bool side via BoolToSigmaProp
            if matches!(hir.op.node, BinaryOp::And | BinaryOp::Or) {
                let l_sigma = l.tpe() == SType::SSigmaProp;
                let r_sigma = r.tpe() == SType::SSigmaProp;
                if l_sigma || r_sigma {
                    let l = if !l_sigma {
                        Expr::BoolToSigmaProp(BoolToSigmaProp::try_build(l).map_err(|e| {
                            MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                        })?)
                    } else {
                        l
                    };
                    let r = if !r_sigma {
                        Expr::BoolToSigmaProp(BoolToSigmaProp::try_build(r).map_err(|e| {
                            MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                        })?)
                    } else {
                        r
                    };
                    return Ok(match hir.op.node {
                        BinaryOp::And => SigmaAnd::new(vec![l, r])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into(),
                        BinaryOp::Or => SigmaOr::new(vec![l, r])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into(),
                        _ => unreachable!(),
                    });
                }
            }
            {
                // Auto-upcast: when mixing numeric types in arithmetic/comparison,
                // upcast the narrower operand to match the wider one.
                // This matches the Scala ErgoScript compiler's implicit conversions.
                let (l, r) = numeric_upcast_pair(l, r);
                if let Some(ty) = check_byte_short_overflow(&hir.op.node, &l, &r) {
                    return Err(MirLoweringError::new(
                        format!("{} overflow", ty),
                        hir_expr.span,
                    ));
                }
                if let Some(folded) = fold_compare_const_const(&hir.op.node, &l, &r) {
                    folded
                } else if let Some(folded) = fold_arith_const_const(&hir.op.node, &l, &r) {
                    folded
                } else if let Some(folded) = fold_eq_neq(&hir.op.node, &l, &r) {
                    folded
                } else {
                    BinOp {
                        kind: hir.op.node.clone().into(),
                        left: l.into(),
                        right: r.into(),
                    }
                    .into()
                }
            }
        }
        hir::ExprKind::Literal(hir) => {
            let constant: Constant = match hir {
                hir::Literal::Int(v) => (*v).into(),
                hir::Literal::Long(v) => (*v).into(),
                hir::Literal::Bool(v) => (*v).into(),
                hir::Literal::String(_) => {
                    // String literals are only used in fromBase16/fromBase58 — they shouldn't reach MIR directly
                    return Err(MirLoweringError::new(
                        "String literal cannot be used directly; use fromBase16()".to_string(),
                        hir_expr.span,
                    ));
                }
            };
            constant.into()
        }
        hir::ExprKind::Apply(apply) => {
            // Check if this is a named built-in function (before lowering func)
            if let hir::ExprKind::Ident(name) = &apply.func.kind {
                // Special handling for fromBase16 — extract string at compile time
                if name == "fromBase16" {
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "fromBase16 requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let hex_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "fromBase16 argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    let bytes = base16::decode(hex_str.as_bytes()).map_err(|e| {
                        MirLoweringError::new(
                            format!("Invalid hex in fromBase16: {:?}", e),
                            hir_expr.span,
                        )
                    })?;
                    return Ok(Constant::from(bytes).into());
                }
                // Special handling for bigInt("...") — parse decimal string to BigInt256.
                if name == "bigInt" {
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "bigInt requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let dec_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "bigInt argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    use num_traits::Num as _;
                    let v = ergotree_ir::bigint256::BigInt256::from_str_radix(dec_str.trim(), 10)
                        .map_err(|e| {
                        MirLoweringError::new(
                            format!("Invalid decimal in bigInt({:?}): {}", dec_str, e),
                            hir_expr.span,
                        )
                    })?;
                    return Ok(Constant::from(v).into());
                }
                // Special handling for unsignedBigInt("...") — parse decimal string to UnsignedBigInt256.
                if name == "unsignedBigInt" {
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "unsignedBigInt requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let dec_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "unsignedBigInt argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    use num_traits::Num as _;
                    let v = ergotree_ir::unsignedbigint256::UnsignedBigInt::from_str_radix(
                        dec_str.trim(),
                        10,
                    )
                    .map_err(|e| {
                        MirLoweringError::new(
                            format!("Invalid decimal in unsignedBigInt({:?}): {}", dec_str, e),
                            hir_expr.span,
                        )
                    })?;
                    return Ok(Constant::from(v).into());
                }
                // Special handling for fromBase64 — decode Base64 string at compile time
                if name == "fromBase64" {
                    use base64::Engine as _;
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "fromBase64 requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let b64_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "fromBase64 argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(b64_str.as_bytes())
                        .map_err(|e| {
                            MirLoweringError::new(
                                format!("Invalid Base64 in fromBase64: {:?}", e),
                                hir_expr.span,
                            )
                        })?;
                    return Ok(Constant::from(bytes).into());
                }
                // Special handling for PK("addr") — parse Ergo P2PK address at compile
                // time and emit a Const(SSigmaProp) carrying the embedded ProveDlog.
                // Mirrors Scala's `PK` predef: a P2PK address's content bytes are the
                // serialized GroupElement of the public key, wrapped as a SigmaProp.
                if name == "PK" {
                    use ergotree_ir::chain::address::{Address, AddressEncoder};
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "PK requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let addr_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "PK argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    let address = AddressEncoder::unchecked_parse_address_from_str(addr_str.trim())
                        .map_err(|e| {
                            MirLoweringError::new(
                                format!("Invalid address in PK({:?}): {}", addr_str, e),
                                hir_expr.span,
                            )
                        })?;
                    let prove_dlog = match address {
                        Address::P2Pk(pd) => pd,
                        _ => {
                            return Err(MirLoweringError::new(
                                format!("PK requires a P2PK address; {:?} is not P2PK", addr_str),
                                hir_expr.span,
                            ))
                        }
                    };
                    return Ok(Constant::from(prove_dlog).into());
                }
                // Special handling for deserialize[T]("base58") — compile-time
                // base58 decode + sigma_parse + type-check, producing the
                // resulting Expr inline. Mirrors Scala's `DeserializeFunc`
                // (SigmaPredef.scala:169).
                if name == "deserialize" {
                    let target = apply.type_arg.clone().ok_or_else(|| {
                        MirLoweringError::new(
                            "deserialize requires a type argument like deserialize[Long](\"...\")"
                                .to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "deserialize requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let b58_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "deserialize argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    let bytes = bs58::decode(&b58_str).into_vec().map_err(|e| {
                        MirLoweringError::new(
                            format!("Invalid Base58 in deserialize: {:?}", e),
                            hir_expr.span,
                        )
                    })?;
                    use ergotree_ir::serialization::SigmaSerializable;
                    let parsed = Expr::sigma_parse_bytes(&bytes).map_err(|e| {
                        MirLoweringError::new(
                            format!("deserialize: sigma_parse failed: {:?}", e),
                            hir_expr.span,
                        )
                    })?;
                    if parsed.tpe() != target {
                        return Err(MirLoweringError::new(
                            format!(
                                "deserialize: deserialized type {:?} does not match expected {:?}",
                                parsed.tpe(),
                                target
                            ),
                            hir_expr.span,
                        ));
                    }
                    return Ok(parsed);
                }
                // Special handling for fromBase58 — decode Base58 string at compile time
                if name == "fromBase58" {
                    let str_arg = apply.args.first().ok_or_else(|| {
                        MirLoweringError::new(
                            "fromBase58 requires a string argument".to_string(),
                            hir_expr.span,
                        )
                    })?;
                    let b58_str = match &str_arg.kind {
                        hir::ExprKind::Literal(hir::Literal::String(s)) => s.clone(),
                        _ => {
                            return Err(MirLoweringError::new(
                                "fromBase58 argument must be a string literal".to_string(),
                                hir_expr.span,
                            ))
                        }
                    };
                    let bytes = bs58::decode(&b58_str).into_vec().map_err(|e| {
                        MirLoweringError::new(
                            format!("Invalid Base58 in fromBase58: {:?}", e),
                            hir_expr.span,
                        )
                    })?;
                    return Ok(Constant::from(bytes).into());
                }
                // Lower all args for other builtins
                let args: Result<Vec<Expr>, MirLoweringError> =
                    apply.args.iter().map(|a| lower(a.clone())).collect();
                let args = args?;
                match name.as_str() {
                    "sigmaProp" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "sigmaProp requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        // If the input is already SSigmaProp, sigmaProp is
                        // a no-op (Scala compiler elides it).
                        if input.tpe() == SType::SSigmaProp {
                            input
                        } else {
                            BoolToSigmaProp {
                                input: input.into(),
                            }
                            .into()
                        }
                    }
                    "ZKProof" => {
                        // ZKProof { sigmaPropExpr } → ZkProofBlock { input: SigmaProp }: SBoolean.
                        // Mirrors Scala's SigmaPredef.ZKProofFunc (irBuilder calls
                        // mkZKProofBlock(block.body)). The Rust parser handles the
                        // `name { ... }` syntax as a FuncCall with the block expr as
                        // a single arg, so the existing predef dispatch is the right
                        // home for it. Frontend-only — serialization and eval both
                        // error to match Scala parity (Scala `OpCodes.Undefined` +
                        // `testMissingCostingWOSerialization`).
                        use ergotree_ir::mir::zk_proof::ZkProofBlock;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "ZKProof requires a single block argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ZkProofBlock::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "blake2b256" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "blake2b256 requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        CalcBlake2b256 {
                            input: input.into(),
                        }
                        .into()
                    }
                    "sha256" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "sha256 requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        CalcSha256::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "proveDlog" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "proveDlog requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        CreateProveDlog {
                            input: input.into(),
                        }
                        .into()
                    }
                    "proveDHTuple" => {
                        let mut it = args.into_iter();
                        let g = it.next();
                        let h = it.next();
                        let u = it.next();
                        let v = it.next();
                        let (g, h, u, v) = match (g, h, u, v) {
                            (Some(g), Some(h), Some(u), Some(v)) => (g, h, u, v),
                            _ => {
                                return Err(MirLoweringError::new(
                                    "proveDHTuple requires four arguments (g, h, u, v)".to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        CreateProveDhTuple::new(g, h, u, v)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "atLeast" => {
                        let mut it = args.into_iter();
                        let bound = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "atLeast requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let input = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "atLeast requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ergotree_ir::mir::atleast::Atleast::new(bound, input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "allOf" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "allOf requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        // Mirror Scala graph-IR fold: AND(BoolConstants[lit*]) collapses
                        // to a single Const(all-true) at build time. Without this,
                        // downstream serialization keeps the COLL_OF_BOOL_CONST + AND
                        // opcodes while Scala emits a bare Const, diverging the bytes.
                        if let Expr::Collection(
                            ergotree_ir::mir::collection::Collection::BoolConstants(ref bools),
                        ) = input
                        {
                            let folded: bool = bools.iter().all(|b| *b);
                            Constant::from(folded).into()
                        } else {
                            ergotree_ir::mir::and::And {
                                input: input.into(),
                            }
                            .into()
                        }
                    }
                    "anyOf" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "anyOf requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        // Mirror Scala graph-IR fold: OR(BoolConstants[lit*]) collapses
                        // to a single Const(any-true) at build time.
                        if let Expr::Collection(
                            ergotree_ir::mir::collection::Collection::BoolConstants(ref bools),
                        ) = input
                        {
                            let folded: bool = bools.iter().any(|b| *b);
                            Constant::from(folded).into()
                        } else {
                            ergotree_ir::mir::or::Or {
                                input: input.into(),
                            }
                            .into()
                        }
                    }
                    "allZK" | "anyZK" => {
                        // Coll-of-SigmaProp version of allOf/anyOf. Scala marks
                        // the irBuilder as `undefined`; the actual lowering happens
                        // in graph-IR via the AllZk/AnyZk extractors which only
                        // match literal Collection.fromItems shapes. Mirror that:
                        // require a literal Coll(...) of SigmaProp items, unfold
                        // to SigmaAnd/SigmaOr (≥2 items required by IR).
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                format!("{} requires one argument", name),
                                hir_expr.span,
                            )
                        })?;
                        let items = match input {
                            Expr::Collection(coll) => match coll {
                                ergotree_ir::mir::collection::Collection::Exprs {
                                    elem_tpe: _,
                                    items,
                                } => items,
                                ergotree_ir::mir::collection::Collection::BoolConstants(_) => {
                                    return Err(MirLoweringError::new(
                                        format!(
                                            "{}: argument must be Coll[SigmaProp], got Coll[Boolean]",
                                            name
                                        ),
                                        hir_expr.span,
                                    ));
                                }
                            },
                            _ => {
                                return Err(MirLoweringError::new(
                                    format!(
                                        "{} requires a literal Coll(...) of SigmaProp items — runtime collections are not supported (matches Scala's `undefined` irBuilder)",
                                        name
                                    ),
                                    hir_expr.span,
                                ));
                            }
                        };
                        if items.len() < 2 {
                            return Err(MirLoweringError::new(
                                format!(
                                    "{} requires at least 2 SigmaProp items, got {}",
                                    name,
                                    items.len()
                                ),
                                hir_expr.span,
                            ));
                        }
                        if name == "allZK" {
                            ergotree_ir::mir::sigma_and::SigmaAnd::new(items)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        } else {
                            ergotree_ir::mir::sigma_or::SigmaOr::new(items)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                    }
                    "longToByteArray" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "longToByteArray requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        LongToByteArray {
                            input: input.into(),
                        }
                        .into()
                    }
                    "min" => {
                        let mut it = args.into_iter();
                        let left = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "min requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let right = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "min requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        // Mirror Scala graph-IR fold: OrderingMin(Const, Const)
                        // collapses to Const at build time via propagateBinOp
                        // (DefRewriting.scala default arm). Helper covers
                        // Byte/Short/Int/Long; BigInt falls through unfolded.
                        if let Some(folded) = fold_min_max_const_const(true, &left, &right) {
                            folded
                        } else {
                            BinOp {
                                kind: ArithOp::Min.into(),
                                left: left.into(),
                                right: right.into(),
                            }
                            .into()
                        }
                    }
                    "Coll" => {
                        // Coll[Type]() or Coll(items...) — collection constructor
                        let elem_tpe = apply.type_arg.clone().unwrap_or_else(|| {
                            args.first().map(|a| a.tpe()).unwrap_or(SType::SByte)
                        });
                        return Ok(Collection::new(elem_tpe, args)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into());
                    }
                    "getVar" => {
                        // getVar[Type](varId) — context variable access
                        let var_id_expr = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "getVar requires a variable ID argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let var_id = match &var_id_expr {
                            Expr::Const(c) => {
                                use ergotree_ir::mir::constant::Literal;
                                match &c.v {
                                    Literal::Int(id) => *id as u8,
                                    Literal::Byte(id) => *id as u8,
                                    _ => {
                                        return Err(MirLoweringError::new(
                                            "getVar variable ID must be an integer constant"
                                                .to_string(),
                                            hir_expr.span,
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "getVar variable ID must be a constant".to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        // Type from [Type] annotation
                        let var_tpe = apply.type_arg.clone().unwrap_or(SType::SAny);
                        return Ok(GetVar { var_id, var_tpe }.into());
                    }
                    "decodePoint" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "decodePoint requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        DecodePoint::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "getVarFromInput" => {
                        // getVarFromInput[T](inputIdx, varId) — Context method (v6.0+).
                        // Lowers to MethodCall on Expr::Context with explicit type
                        // arg substituting STypeVar::t(). The IR signature requires
                        // (SShort, SByte); we accept SInt literals from the user
                        // source and Downcast them.
                        use ergotree_ir::types::scontext::GET_VAR_FROM_INPUT_METHOD;
                        use ergotree_ir::types::stype_param::STypeVar;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "getVarFromInput requires a type argument like getVarFromInput[Long](0, 1)".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let mut it = args.into_iter();
                        let input_idx = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "getVarFromInput requires (inputIdx, varId) arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let var_id = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "getVarFromInput requires (inputIdx, varId) arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let coerce_to =
                            |e: Expr, target: SType| -> Result<Expr, MirLoweringError> {
                                if e.tpe() == target {
                                    return Ok(e);
                                }
                                Downcast::new(e, target.clone())
                                    .map(Into::into)
                                    .map_err(|err| {
                                        MirLoweringError::new(
                                            format!(
                                                "getVarFromInput coercion to {:?}: {:?}",
                                                target, err
                                            ),
                                            hir_expr.span,
                                        )
                                    })
                            };
                        let input_idx = coerce_to(input_idx, SType::SShort)?;
                        let var_id = coerce_to(var_id, SType::SByte)?;
                        let mut type_args: hashbrown::HashMap<STypeVar, SType> =
                            hashbrown::HashMap::new();
                        type_args.insert(STypeVar::t(), target);
                        MethodCall::with_type_args(
                            Expr::Context,
                            GET_VAR_FROM_INPUT_METHOD.clone(),
                            vec![input_idx, var_id],
                            type_args,
                        )
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                    }
                    "placeholder" => {
                        // placeholder[T](id: Int) → T.
                        // Lowers to ConstantPlaceholder { id, tpe: T }. Internal
                        // ErgoTree primitive used for constant segregation; user
                        // code calling it produces the bare placeholder node and
                        // is responsible for providing a constants array at
                        // serialization time. Scala's `placeholder` predef
                        // (SigmaPredef.scala:730) ships an `undefined` irBuilder
                        // and is parser-only there; the Rust IR has the node, so
                        // we surface a wiring for completeness.
                        use ergotree_ir::mir::constant::ConstantPlaceholder;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "placeholder requires a type argument like placeholder[Long](0)"
                                    .to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let id_expr = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "placeholder requires an id argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let id: u32 = match &id_expr {
                            Expr::Const(c) => {
                                use ergotree_ir::mir::constant::Literal;
                                match &c.v {
                                    Literal::Int(v) if *v >= 0 => *v as u32,
                                    Literal::Byte(v) if *v >= 0 => *v as u32,
                                    _ => return Err(MirLoweringError::new(
                                        "placeholder id must be a non-negative integer constant"
                                            .to_string(),
                                        hir_expr.span,
                                    )),
                                }
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "placeholder id must be a constant".to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        Expr::ConstPlaceholder(ConstantPlaceholder { id, tpe: target })
                    }
                    "executeFromVar" => {
                        // executeFromVar[T](id: Byte) → T.
                        // Lowers to DeserializeContext { tpe: T, id: u8 }.
                        // Mirrors Scala's mkDeserializeContext.
                        use ergotree_ir::mir::deserialize_context::DeserializeContext;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromVar requires a type argument like executeFromVar[Long](0)".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let id_expr = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromVar requires an id argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let id = match &id_expr {
                            Expr::Const(c) => {
                                use ergotree_ir::mir::constant::Literal;
                                match &c.v {
                                    Literal::Int(v) => *v as u8,
                                    Literal::Byte(v) => *v as u8,
                                    _ => {
                                        return Err(MirLoweringError::new(
                                            "executeFromVar id must be an integer constant"
                                                .to_string(),
                                            hir_expr.span,
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "executeFromVar id must be a constant".to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        DeserializeContext { tpe: target, id }.into()
                    }
                    "executeFromSelfReg" => {
                        // executeFromSelfReg[T](id: Int) → T.
                        // Lowers to DeserializeRegister { reg, tpe: T, default: None }.
                        // Mirrors Scala's mkDeserializeRegister(r, rtpe, None).
                        use ergotree_ir::chain::ergo_box::RegisterId;
                        use ergotree_ir::mir::deserialize_register::DeserializeRegister;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromSelfReg requires a type argument like executeFromSelfReg[Long](4)".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let id_expr = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromSelfReg requires an id argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let idx: i32 = match &id_expr {
                            Expr::Const(c) => {
                                use ergotree_ir::mir::constant::Literal;
                                match &c.v {
                                    Literal::Int(v) => *v,
                                    Literal::Byte(v) => *v as i32,
                                    _ => {
                                        return Err(MirLoweringError::new(
                                            "executeFromSelfReg id must be an integer constant"
                                                .to_string(),
                                            hir_expr.span,
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "executeFromSelfReg id must be a constant".to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        let reg = i8::try_from(idx)
                            .ok()
                            .and_then(|v| RegisterId::try_from(v).ok())
                            .ok_or_else(|| {
                                MirLoweringError::new(
                                    format!(
                                        "executeFromSelfReg id {} is out of bounds (0..=9)",
                                        idx
                                    ),
                                    hir_expr.span,
                                )
                            })?;
                        DeserializeRegister::new(reg, target, None)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "executeFromSelfRegWithDefault" => {
                        // executeFromSelfRegWithDefault[T](id: Int, default: T) → T.
                        // Lowers to DeserializeRegister { reg, tpe: T, default: Some(default) }.
                        // Mirrors Scala's mkDeserializeRegister(r, rtpe, Some(default)).
                        use ergotree_ir::chain::ergo_box::RegisterId;
                        use ergotree_ir::mir::deserialize_register::DeserializeRegister;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromSelfRegWithDefault requires a type argument"
                                    .to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let mut it = args.into_iter();
                        let id_expr = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromSelfRegWithDefault requires (id, default) arguments"
                                    .to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let default_expr = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "executeFromSelfRegWithDefault requires (id, default) arguments"
                                    .to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let idx: i32 = match &id_expr {
                            Expr::Const(c) => {
                                use ergotree_ir::mir::constant::Literal;
                                match &c.v {
                                    Literal::Int(v) => *v,
                                    Literal::Byte(v) => *v as i32,
                                    _ => {
                                        return Err(MirLoweringError::new(
                                            "executeFromSelfRegWithDefault id must be an integer constant".to_string(),
                                            hir_expr.span,
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "executeFromSelfRegWithDefault id must be a constant"
                                        .to_string(),
                                    hir_expr.span,
                                ))
                            }
                        };
                        let reg = i8::try_from(idx)
                            .ok()
                            .and_then(|v| RegisterId::try_from(v).ok())
                            .ok_or_else(|| {
                                MirLoweringError::new(
                                    format!("executeFromSelfRegWithDefault id {} is out of bounds (0..=9)", idx),
                                    hir_expr.span,
                                )
                            })?;
                        DeserializeRegister::new(reg, target, Some(Box::new(default_expr)))
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "some" => {
                        // some(value) — Global.some(value): Option[T].
                        // T is inferred from the argument's type via specialize_for.
                        // V3+ method.
                        use ergotree_ir::types::sglobal::SOME_METHOD;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "some requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let specialized = SOME_METHOD
                            .clone()
                            .specialize_for(SType::SGlobal, vec![input.tpe()])
                            .map_err(|e| {
                                MirLoweringError::new(
                                    format!("some specialize: {:?}", e),
                                    hir_expr.span,
                                )
                            })?;
                        MethodCall::new(Expr::Global, specialized, vec![input])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "none" => {
                        // none[T]() — Global.none[T]: Option[T].
                        // Requires explicit type arg; explicit_type_args = [STypeVar::t()].
                        // V3+ method.
                        use ergotree_ir::types::sglobal::NONE_METHOD;
                        use ergotree_ir::types::stype_param::STypeVar;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "none requires a type argument like none[Long]()".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let mut type_args: hashbrown::HashMap<STypeVar, SType> =
                            hashbrown::HashMap::new();
                        type_args.insert(STypeVar::t(), target);
                        MethodCall::with_type_args(
                            Expr::Global,
                            NONE_METHOD.clone(),
                            vec![],
                            type_args,
                        )
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                    }
                    "serialize" => {
                        // serialize[T](value) — Global.serialize, T is inferred
                        // from the argument type via SMethod::specialize_for.
                        use ergotree_ir::types::sglobal::SERIALIZE_METHOD;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "serialize requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let specialized = SERIALIZE_METHOD
                            .clone()
                            .specialize_for(SType::SGlobal, vec![input.tpe()])
                            .map_err(|e| {
                                MirLoweringError::new(
                                    format!("serialize specialize: {:?}", e),
                                    hir_expr.span,
                                )
                            })?;
                        MethodCall::new(Expr::Global, specialized, vec![input])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "deserializeTo" => {
                        // deserializeTo[T](bytes) — Global.deserialize (v6.0+).
                        // The user-facing name is `deserializeTo` per the EKB
                        // built-ins ref; the underlying IR method is named
                        // `deserialize` with an explicit type arg.
                        use ergotree_ir::types::sglobal::DESERIALIZE_METHOD;
                        use ergotree_ir::types::stype_param::STypeVar;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "deserializeTo requires a type argument like deserializeTo[Long](bytes)".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "deserializeTo requires a Coll[Byte] argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let mut type_args: hashbrown::HashMap<STypeVar, SType> =
                            hashbrown::HashMap::new();
                        type_args.insert(STypeVar::t(), target);
                        MethodCall::with_type_args(
                            Expr::Global,
                            DESERIALIZE_METHOD.clone(),
                            vec![input],
                            type_args,
                        )
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                    }
                    "encodeNbits" => {
                        use ergotree_ir::types::sglobal::ENCODE_NBITS_METHOD;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "encodeNbits requires one BigInt argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        MethodCall::new(Expr::Global, ENCODE_NBITS_METHOD.clone(), vec![input])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "decodeNbits" => {
                        use ergotree_ir::types::sglobal::DECODE_NBITS_METHOD;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "decodeNbits requires one Long argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        MethodCall::new(Expr::Global, DECODE_NBITS_METHOD.clone(), vec![input])
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "powHit" => {
                        // powHit(k, msg, nonce, h, N) → Boolean. Global method (v6.0+).
                        use ergotree_ir::types::sglobal::POW_HIT_METHOD;
                        if args.len() != 5 {
                            return Err(MirLoweringError::new(
                                format!("powHit requires 5 arguments, got {}", args.len()),
                                hir_expr.span,
                            ));
                        }
                        MethodCall::new(Expr::Global, POW_HIT_METHOD.clone(), args)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "fromBigEndianBytes" => {
                        // fromBigEndianBytes[T](bytes) — Global method call (v6.0+).
                        // The user-facing form omits the `Global.` receiver; we
                        // re-attach it here. The explicit type arg substitutes
                        // STypeVar::t() in the method's t_range.
                        use ergotree_ir::types::sglobal::FROM_BIGENDIAN_BYTES_METHOD;
                        use ergotree_ir::types::stype_param::STypeVar;
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "fromBigEndianBytes requires a type argument like fromBigEndianBytes[Long](bytes)".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "fromBigEndianBytes requires a Coll[Byte] argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let mut type_args: hashbrown::HashMap<STypeVar, SType> =
                            hashbrown::HashMap::new();
                        type_args.insert(STypeVar::t(), target);
                        MethodCall::with_type_args(
                            Expr::Global,
                            FROM_BIGENDIAN_BYTES_METHOD.clone(),
                            vec![input],
                            type_args,
                        )
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                    }
                    "max" => {
                        let mut it = args.into_iter();
                        let left = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "max requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let right = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "max requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        // See "min" arm above for Scala graph-IR fold rationale.
                        if let Some(folded) = fold_min_max_const_const(false, &left, &right) {
                            folded
                        } else {
                            BinOp {
                                kind: ArithOp::Max.into(),
                                left: left.into(),
                                right: right.into(),
                            }
                            .into()
                        }
                    }
                    "substConstants" => {
                        let mut it = args.into_iter();
                        let script_bytes = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "substConstants requires three arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let positions = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "substConstants requires three arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let new_values = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "substConstants requires three arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        SubstConstants::new(script_bytes, positions, new_values)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "byteArrayToLong" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "byteArrayToLong requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ByteArrayToLong::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "byteArrayToBigInt" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "byteArrayToBigInt requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ByteArrayToBigInt::try_build(input)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "xor" => {
                        let mut it = args.into_iter();
                        let left = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "xor requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let right = it.next().ok_or_else(|| {
                            MirLoweringError::new(
                                "xor requires two arguments".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        Xor::new(left, right)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "avlTree" => {
                        // avlTree(operationFlags: Byte, digest: Coll[Byte], keyLength: Int,
                        //         valueLengthOpt: Option[Int]) → AvlTree
                        //
                        // NOTE: Scala IR's CreateAvlTree takes a runtime Option-typed Expr
                        // for valueLengthOpt; Rust's `CreateAvlTree::value_length` is a
                        // compile-time `Option<Box<Expr>>` of the inner Int Expr. To bridge
                        // this, we pattern-match on the 4th arg's lowered MIR shape:
                        //   `none[Int]()` → Rust None
                        //   `some(intExpr)` → Rust Some(intExpr)
                        // Other shapes (runtime Option-typed values) would need an IR fix
                        // first and are rejected with an explanatory error. Byte-match
                        // against Scala for this predef is a known IR-level discrepancy;
                        // none of the 14/14 ecosystem fixtures exercise this path.
                        use ergotree_ir::mir::create_avl_tree::CreateAvlTree;
                        if args.len() != 4 {
                            return Err(MirLoweringError::new(
                                format!(
                                    "avlTree requires (flags, digest, keyLength, valueLengthOpt) — got {} args",
                                    args.len()
                                ),
                                hir_expr.span,
                            ));
                        }
                        let mut it = args.into_iter();
                        let flags = it.next().unwrap();
                        let digest = it.next().unwrap();
                        let key_length = it.next().unwrap();
                        let raw_vlen = it.next().unwrap();
                        let value_length: Option<Box<Expr>> = match raw_vlen {
                            Expr::MethodCall(ref mc) if mc.expr.method.name() == "none" => None,
                            Expr::MethodCall(mc) if mc.expr.method.name() == "some" => {
                                let inner = mc.expr.args.into_iter().next().ok_or_else(|| {
                                    MirLoweringError::new(
                                        "avlTree: malformed some(_) for valueLengthOpt".to_string(),
                                        hir_expr.span,
                                    )
                                })?;
                                Some(Box::new(inner))
                            }
                            _ => {
                                return Err(MirLoweringError::new(
                                    "avlTree's valueLengthOpt must be a literal `some(n)` or `none[Int]()` — runtime Option-typed expressions are not supported"
                                        .to_string(),
                                    hir_expr.span,
                                ));
                            }
                        };
                        CreateAvlTree::new(flags, digest, key_length, value_length)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "treeLookup" => {
                        // treeLookup(tree, key, proof) → Option[Coll[Byte]]
                        if args.len() != 3 {
                            return Err(MirLoweringError::new(
                                format!(
                                    "treeLookup requires (tree, key, proof) — got {} args",
                                    args.len()
                                ),
                                hir_expr.span,
                            ));
                        }
                        let mut it = args.into_iter();
                        let tree = it.next().unwrap();
                        let key = it.next().unwrap();
                        let proof = it.next().unwrap();
                        TreeLookup::new(tree, key, proof)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "upcast" => {
                        // upcast[T](x) — explicit numeric widening cast.
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "upcast requires a type argument like upcast[Long](x)".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "upcast requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        Upcast::new(input, target)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "downcast" => {
                        // downcast[T](x) — explicit numeric narrowing cast.
                        let target = apply.type_arg.clone().ok_or_else(|| {
                            MirLoweringError::new(
                                "downcast requires a type argument like downcast[Byte](x)"
                                    .to_string(),
                                hir_expr.span,
                            )
                        })?;
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "downcast requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        Downcast::new(input, target)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    "xorOf" => {
                        let input = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "xorOf requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        XorOf {
                            input: input.into(),
                        }
                        .into()
                    }
                    other => {
                        return Err(MirLoweringError::new(
                            format!("MIR error: Unknown function: {}", other),
                            hir_expr.span,
                        ))
                    }
                }
            } else if let hir::ExprKind::FieldAccess(fa) = &apply.func.kind {
                // Method call: coll.method(lambda)
                let obj = lower(*fa.object.clone())?;
                let args: Result<Vec<Expr>, MirLoweringError> =
                    apply.args.iter().map(|a| lower(a.clone())).collect();
                let args = args?;
                let method = fa.field.as_str();
                // Check if this is property access + indexing (not a collection method call)
                // e.g., SELF.tokens(0), CONTEXT.dataInputs(0)
                #[allow(clippy::nonminimal_bool)]
                if !matches!(
                    method,
                    "filter"
                        | "exists"
                        | "forall"
                        | "map"
                        | "flatMap"
                        | "fold"
                        | "slice"
                        | "append"
                        | "getOrElse"
                        | "insert"
                        | "update"
                        | "remove"
                        | "getMany"
                        | "contains"
                        | "updateDigest"
                        | "updateOperations"
                        | "exp"
                        | "expUnsigned"
                        | "multiply"
                        | "zip"
                        | "patch"
                        | "updated"
                        | "updateMany"
                        | "indexOf"
                        | "startsWith"
                        | "endsWith"
                        // SNumericTypeMethods V6 (with args)
                        | "bitwiseOr"
                        | "bitwiseAnd"
                        | "bitwiseXor"
                        | "shiftLeft"
                        | "shiftRight"
                        | "toUnsignedMod"
                        | "modInverse"
                        | "plusMod"
                        | "subtractMod"
                        | "multiplyMod"
                        | "mod"
                        // SAvlTreeMethods (V6)
                        | "insertOrUpdate"
                ) && !(method == "get"
                    && matches!(fa.object.tpe, Some(SType::SAvlTree | SType::SColl(_))))
                {
                    // Lower the FieldAccess as a property first, then apply indexing
                    let prop = lower(*apply.func.clone())?;
                    let args: Result<Vec<Expr>, MirLoweringError> =
                        apply.args.iter().map(|a| lower(a.clone())).collect();
                    let args = args?;
                    match prop.tpe() {
                        SType::SColl(_) => {
                            let index = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "Collection index requires one argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ByIndex::new(prop, upcast_index_to_int(index), None)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        _ => {
                            return Err(MirLoweringError::new(
                                "MIR error: Cannot index non-collection".to_string(),
                                hir_expr.span,
                            ))
                        }
                    }
                } else {
                    match method {
                        "filter" if matches!(obj.tpe(), SType::SOption(_)) => {
                            // Option[T].filter((T) → Bool) → Option[T]
                            use ergotree_ir::types::soption::FILTER_METHOD;
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "Option.filter requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let specialized = FILTER_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), vec![cond.tpe()])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, vec![cond])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "filter" => {
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "filter requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Filter::new(obj, cond)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "exists" => {
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "exists requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Exists::new(obj, cond)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "forall" => {
                            let cond = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "forall requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ForAll::new(obj, cond)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "fold" => {
                            let mut it = args.into_iter();
                            let zero = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "fold requires zero value".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let fold_op = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "fold requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let fold_op = transform_fold_lambda(fold_op, hir_expr.span)?;
                            Fold::new(obj, zero, fold_op)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "map" if matches!(obj.tpe(), SType::SOption(_)) => {
                            // Option[T].map((T) → U) → Option[U]
                            use ergotree_ir::types::soption::MAP_METHOD;
                            let mapper = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "Option.map requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let specialized = MAP_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), vec![mapper.tpe()])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, vec![mapper])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "map" => {
                            let mapper = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "map requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Map::new(obj, mapper)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "append" => {
                            let col2 = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "append requires a collection argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Append::new(obj, col2)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "flatMap" => {
                            let mapper = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "flatMap requires a lambda".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            use ergotree_ir::types::scoll::FLATMAP_METHOD;
                            // Specialize generic FLATMAP_METHOD with concrete types
                            let specialized = FLATMAP_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), vec![mapper.tpe()])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, vec![mapper])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        // AvlTree methods
                        "get" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                            let mut it = args.into_iter();
                            let key = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "AvlTree.get requires key".into(),
                                    hir_expr.span,
                                )
                            })?;
                            let proof = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "AvlTree.get requires proof".into(),
                                    hir_expr.span,
                                )
                            })?;
                            // Lower to MethodCall(get) — Scala's TreeBuilding emits
                            // this as MethodCall (opcode 0xdc) rather than the
                            // dedicated TreeLookup opcode (0xb7). Matches NODE
                            // byte encoding for chaincash AvlTree.get.
                            use ergotree_ir::types::savltree::GET_METHOD;
                            MethodCall::new(obj, GET_METHOD.clone(), vec![key, proof])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "exp" => {
                            // SGroupElement.exp(scalar) → Exponentiate
                            let scalar = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "exp requires one scalar argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Exponentiate::new(obj, scalar)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "multiply" => {
                            // SGroupElement.multiply(other) → MultiplyGroup (dedicated opcode,
                            // mirrors `exp` → Exponentiate above). MethodCall path is rejected
                            // by the deserializer because GroupElement.multiply (MethodId 4) is
                            // not registered in METHOD_DESC at sgroup_elem.rs.
                            let other = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "multiply requires one GroupElement argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ergotree_ir::mir::multiply_group::MultiplyGroup::new(obj, other)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "expUnsigned" if matches!(fa.object.tpe, Some(SType::SGroupElement)) => {
                            // SGroupElement.expUnsigned(UnsignedBigInt) → MethodCall (V3+)
                            use ergotree_ir::types::sgroup_elem::EXPONENTIATE_UNSIGNED_METHOD;
                            MethodCall::new(obj, EXPONENTIATE_UNSIGNED_METHOD.clone(), args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "insert" | "update" | "remove" | "getMany" | "contains"
                        | "updateDigest" | "updateOperations" => {
                            use ergotree_ir::types::savltree;
                            let method = match method {
                                "insert" => savltree::INSERT_METHOD.clone(),
                                "update" => savltree::UPDATE_METHOD.clone(),
                                "remove" => savltree::REMOVE_METHOD.clone(),
                                "getMany" => savltree::GET_MANY_METHOD.clone(),
                                "contains" => savltree::CONTAINS_METHOD.clone(),
                                "updateDigest" => savltree::UPDATE_DIGEST_METHOD.clone(),
                                "updateOperations" => savltree::UPDATE_OPERATIONS_METHOD.clone(),
                                _ => unreachable!(),
                            };
                            MethodCall::new(obj, method, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "insertOrUpdate" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                            // V6 method: (Coll[(Coll[Byte], Coll[Byte])], Coll[Byte]) → Option[AvlTree]
                            use ergotree_ir::types::savltree::INSERT_OR_UPDATE_METHOD;
                            MethodCall::new(obj, INSERT_OR_UPDATE_METHOD.clone(), args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "getOrElse" if matches!(obj.tpe(), SType::SOption(_)) => {
                            // Option[T].getOrElse(default: T) -> T
                            use ergotree_ir::mir::option_get_or_else::OptionGetOrElse;
                            let default = args.into_iter().next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "Option.getOrElse requires default argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            OptionGetOrElse::new(obj, default)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "getOrElse" => {
                            // Coll[T].getOrElse(index: Int, default: T) -> T
                            let mut it = args.into_iter();
                            let index = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "getOrElse requires index argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let default = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "getOrElse requires default argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            ByIndex::new(obj, upcast_index_to_int(index), Some(default.into()))
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "slice" => {
                            let mut it = args.into_iter();
                            let from = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "slice requires from argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let until = it.next().ok_or_else(|| {
                                MirLoweringError::new(
                                    "slice requires until argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            Slice::new(obj, from, until)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        // SCollection methods (V0)
                        "zip" => {
                            use ergotree_ir::types::scoll::ZIP_METHOD;
                            let other = args.first().cloned().ok_or_else(|| {
                                MirLoweringError::new(
                                    "zip requires a collection argument".to_string(),
                                    hir_expr.span,
                                )
                            })?;
                            let specialized = ZIP_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), vec![other.tpe()])
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "patch" => {
                            use ergotree_ir::types::scoll::PATCH_METHOD;
                            if args.len() != 3 {
                                return Err(MirLoweringError::new(
                                    "patch requires 3 arguments (from, patch, replaced)"
                                        .to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = PATCH_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "updated" => {
                            use ergotree_ir::types::scoll::UPDATED_METHOD;
                            if args.len() != 2 {
                                return Err(MirLoweringError::new(
                                    "updated requires 2 arguments (index, elem)".to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = UPDATED_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "updateMany" => {
                            use ergotree_ir::types::scoll::UPDATE_MANY_METHOD;
                            if args.len() != 2 {
                                return Err(MirLoweringError::new(
                                    "updateMany requires 2 arguments (indices, values)".to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = UPDATE_MANY_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "indexOf" => {
                            use ergotree_ir::types::scoll::INDEX_OF_METHOD;
                            if args.len() != 2 {
                                return Err(MirLoweringError::new(
                                    "indexOf requires 2 arguments (elem, from)".to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = INDEX_OF_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        // SCollection methods (V6)
                        "startsWith" => {
                            use ergotree_ir::types::scoll::STARTS_WITH_METHOD;
                            if args.len() != 1 {
                                return Err(MirLoweringError::new(
                                    "startsWith requires 1 argument (prefix)".to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = STARTS_WITH_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "endsWith" => {
                            use ergotree_ir::types::scoll::ENDS_WITH_METHOD;
                            if args.len() != 1 {
                                return Err(MirLoweringError::new(
                                    "endsWith requires 1 argument (suffix)".to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = ENDS_WITH_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "get" if matches!(fa.object.tpe, Some(SType::SColl(_))) => {
                            // Coll[T].get(idx: Int): Option[T] — V6 method, distinct from
                            // V0 `apply` which throws on out-of-bounds. AvlTree.get is
                            // handled by the prior arm above.
                            use ergotree_ir::types::scoll::GET_METHOD;
                            if args.len() != 1 {
                                return Err(MirLoweringError::new(
                                    "Coll.get requires 1 argument (index)".to_string(),
                                    hir_expr.span,
                                ));
                            }
                            let arg_tpes: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                            let specialized = GET_METHOD
                                .clone()
                                .specialize_for(obj.tpe(), arg_tpes)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?;
                            MethodCall::new(obj, specialized, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        // SNumericTypeMethods V6 — argful methods. All V3+.
                        // bitwiseOr/And/Xor: (T) → T. shiftLeft/Right: (Int) → T.
                        // BigInt only: toUnsignedMod (UnsignedBigInt) → UnsignedBigInt.
                        // UnsignedBigInt only: modInverse / mod (1 arg) and
                        // plusMod/subtractMod/multiplyMod (2 args), all → UnsignedBigInt.
                        "bitwiseOr" | "bitwiseAnd" | "bitwiseXor" | "shiftLeft" | "shiftRight"
                            if is_numeric_tpe(&fa.object.tpe) =>
                        {
                            let smethod =
                                lookup_numeric_method(&obj.tpe(), method).ok_or_else(|| {
                                    MirLoweringError::new(
                                        format!("MIR error: numeric method not found: {}", method),
                                        hir_expr.span,
                                    )
                                })?;
                            MethodCall::new(obj, smethod, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "toUnsignedMod" if matches!(fa.object.tpe, Some(SType::SBigInt)) => {
                            let smethod = lookup_numeric_method(&obj.tpe(), "toUnsignedMod")
                                .ok_or_else(|| {
                                    MirLoweringError::new(
                                        "MIR error: BigInt.toUnsignedMod not found".to_string(),
                                        hir_expr.span,
                                    )
                                })?;
                            MethodCall::new(obj, smethod, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        "modInverse" | "plusMod" | "subtractMod" | "multiplyMod" | "mod"
                            if matches!(fa.object.tpe, Some(SType::SUnsignedBigInt)) =>
                        {
                            let smethod =
                                lookup_numeric_method(&obj.tpe(), method).ok_or_else(|| {
                                    MirLoweringError::new(
                                        format!("MIR error: UnsignedBigInt.{} not found", method),
                                        hir_expr.span,
                                    )
                                })?;
                            MethodCall::new(obj, smethod, args)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                        _ => {
                            return Err(MirLoweringError::new(
                                format!("MIR error: Unknown method: {}", method),
                                hir_expr.span,
                            ))
                        }
                    }
                } // close the else for property-vs-method
            } else {
                // Collection indexing: coll(index)
                let func = lower(*apply.func.clone())?;
                let args: Result<Vec<Expr>, MirLoweringError> =
                    apply.args.iter().map(|a| lower(a.clone())).collect();
                let args = args?;
                match func.tpe() {
                    SType::SColl(_) => {
                        let index = args.into_iter().next().ok_or_else(|| {
                            MirLoweringError::new(
                                "Collection index requires one argument".to_string(),
                                hir_expr.span,
                            )
                        })?;
                        ByIndex::new(func, upcast_index_to_int(index), None)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    SType::SFunc(_) => {
                        // Lambda/function application: f(args)
                        ergotree_ir::mir::apply::Apply::new(func, args)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                    _ => {
                        return Err(MirLoweringError::new(
                            "MIR error: Cannot apply non-function/non-collection".to_string(),
                            hir_expr.span,
                        ))
                    }
                }
            }
        }
        hir::ExprKind::Block(exprs) => {
            if exprs.len() == 1 {
                // Single-expression block: unwrap
                return lower(exprs[0].clone());
            }
            // Multi-expression block: items are ValDefs, last is result
            let last = exprs.last().ok_or_else(|| {
                MirLoweringError::new("Empty block expression".to_string(), hir_expr.span)
            })?;
            let items: Result<Vec<Expr>, MirLoweringError> = exprs[..exprs.len() - 1]
                .iter()
                .map(|e| lower(e.clone()))
                .collect();
            let result = lower(last.clone())?;
            BlockValue {
                items: items?,
                result: result.into(),
            }
            .into()
        }
        hir::ExprKind::ValDef(val_def) => {
            let id = val_def.id.ok_or_else(|| {
                MirLoweringError::new(
                    format!("MIR error: ValDef without id: {}", val_def.name),
                    hir_expr.span,
                )
            })?;
            let rhs = lower(*val_def.rhs.clone())?;
            return Ok(Expr::ValDef(
                ValDef {
                    id: ValId(id),
                    rhs: rhs.into(),
                }
                .into(),
            ));
        }
        hir::ExprKind::ValUse(val_use) => {
            return Ok(Expr::ValUse(ValUse {
                val_id: ValId(val_use.id),
                tpe: val_use.tpe.clone(),
            }));
        }
        hir::ExprKind::Context => {
            return Ok(Expr::Context);
        }
        hir::ExprKind::Tuple(items) => {
            let mir_items: Result<Vec<Expr>, MirLoweringError> =
                items.iter().map(|item| lower(item.clone())).collect();
            return Ok(Tuple::new(mir_items?)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::LogicalNot(inner) => {
            let mir_inner = lower(*inner.clone())?;
            return Ok(LogicalNot::try_build(mir_inner)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::Negation(inner) => {
            let mir_inner = lower(*inner.clone())?;
            return Ok(Negation::try_build(mir_inner)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::BitInversion(inner) => {
            let mir_inner = lower(*inner.clone())?;
            return Ok(BitInversion::try_build(mir_inner)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                .into());
        }
        hir::ExprKind::If(if_expr) => {
            let condition = lower(*if_expr.condition.clone())?;
            let true_branch = lower(*if_expr.then_branch.clone())?;
            let false_branch = lower(*if_expr.else_branch.clone())?;
            return Ok(If {
                condition: condition.into(),
                true_branch: true_branch.into(),
                false_branch: false_branch.into(),
            }
            .into());
        }
        hir::ExprKind::Lambda(lambda) => {
            let args: Vec<FuncArg> = lambda
                .param_ids
                .iter()
                .zip(lambda.params.iter())
                .map(|(id, (_, tpe))| FuncArg {
                    idx: ValId(*id),
                    tpe: tpe.clone(),
                })
                .collect();
            let body = lower(*lambda.body.clone())?;
            return Ok(Expr::FuncValue(FuncValue::new(args, body)));
        }
        hir::ExprKind::FieldAccess(fa) => {
            let obj = lower(*fa.object.clone())?;
            let obj_tpe = fa.object.tpe.clone();
            match fa.field.as_str() {
                "value" => ExtractAmount { input: obj.into() }.into(),
                "propositionBytes" => ExtractScriptBytes { input: obj.into() }.into(),
                "id" if matches!(fa.object.tpe, Some(SType::SBox)) => {
                    ExtractId { input: obj.into() }.into()
                }
                "creationInfo" => ExtractCreationInfo { input: obj.into() }.into(),
                "bytes" => ExtractBytes { input: obj.into() }.into(),
                "bytesWithoutRef" if matches!(fa.object.tpe, Some(SType::SBox)) => {
                    use ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef;
                    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
                    ExtractBytesWithNoRef::try_build(obj)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "size" => {
                    if let Some(folded) = fold_coll_size_on_known_length(&obj) {
                        folded
                    } else {
                        SizeOf { input: obj.into() }.into()
                    }
                }
                "indices" if matches!(fa.object.tpe, Some(SType::SColl(_))) => {
                    use ergotree_ir::types::scoll::INDICES_METHOD;
                    let specialized = INDICES_METHOD
                        .clone()
                        .specialize_for(obj.tpe(), vec![])
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?;
                    PropertyCall::new(obj, specialized)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "reverse" if matches!(fa.object.tpe, Some(SType::SColl(_))) => {
                    use ergotree_ir::types::scoll::REVERSE_METHOD;
                    let specialized = REVERSE_METHOD
                        .clone()
                        .specialize_for(obj.tpe(), vec![])
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?;
                    PropertyCall::new(obj, specialized)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "tokens" => {
                    use ergotree_ir::types::sbox::TOKENS_METHOD;
                    PropertyCall::new(obj, TOKENS_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                field if field.starts_with('_') => {
                    // Tuple field access: _1, _2, etc.
                    let idx: u8 = field[1..].parse().map_err(|_| {
                        MirLoweringError::new(
                            format!("Invalid tuple field: {}", field),
                            hir_expr.span,
                        )
                    })?;
                    let field_index = TupleFieldIndex::try_from(idx).map_err(|_| {
                        MirLoweringError::new(
                            format!("Tuple field index out of bounds: {}", idx),
                            hir_expr.span,
                        )
                    })?;
                    SelectField::new(obj, field_index)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "dataInputs" => {
                    use ergotree_ir::types::scontext::DATA_INPUTS_PROPERTY;
                    PropertyCall::new(obj, DATA_INPUTS_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "selfBoxIndex" => {
                    use ergotree_ir::types::scontext::SELF_BOX_INDEX_PROPERTY;
                    PropertyCall::new(obj, SELF_BOX_INDEX_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "HEIGHT" if matches!(fa.object.tpe, Some(SType::SContext)) => {
                    // CONTEXT.HEIGHT — same as bare HEIGHT global
                    GlobalVars::Height.into()
                }
                "get" => OptionGet::try_build(obj)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "isDefined" => OptionIsDefined::try_build(obj)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "propBytes" => SigmaPropBytes::try_build(obj)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                    .into(),
                "isProven" if matches!(fa.object.tpe, Some(SType::SSigmaProp)) => {
                    SigmaPropIsProven::try_build(obj)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "toLong" => {
                    if let Some(folded) = fold_int_lit_to_long(&obj) {
                        folded
                    } else {
                        Upcast::new(obj, SType::SLong)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "toBigInt" => {
                    // No-op: already BigInt
                    if obj.tpe() == SType::SBigInt {
                        obj
                    } else if let Some(folded) = fold_to_bigint_on_const(&obj) {
                        // Explicit `intLit.toBigInt` — fold to `Const(SBigInt)` at
                        // graph-build time, mirroring Scala. This is distinct from the
                        // implicit `numeric_upcast` BinOp-coercion path (kept unfolded
                        // so Site 1 can strip the Upcast wrapper and let the constant
                        // segregate as its source-level numeric type, e.g. spectrum
                        // FeeDenom). Scala's serializer keeps explicit-toBigInt folds
                        // even for pre-v3 trees because the node is a bare Const by
                        // serialization time, not an Upcast wrapper, so Site 1's
                        // Upcast-on-Const strip never fires on this path.
                        folded
                    } else {
                        Upcast::new(obj, SType::SBigInt)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "toInt" => {
                    // Upcast for smaller→Int, Downcast for larger→Int.
                    // Scala folds Downcast on a literal to a Const of the target
                    // type at graph-build time (see TreeBuilding / graph IR);
                    // mirror that to avoid a residual Downcast wrapper on
                    // constants. Scala does NOT fold Upcast on a literal
                    // (except the existing toBigInt special case), so the
                    // Upcast branch stays unfolded.
                    if matches!(obj.tpe(), SType::SLong | SType::SBigInt) {
                        if let Some(folded) = fold_numeric_downcast_on_const(&obj, SType::SInt) {
                            folded
                        } else {
                            Downcast::new(obj, SType::SInt)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                    } else {
                        Upcast::new(obj, SType::SInt)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "toByte" => {
                    if let Some(folded) = fold_numeric_downcast_on_const(&obj, SType::SByte) {
                        folded
                    } else {
                        Downcast::new(obj, SType::SByte)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                "toShort" => {
                    if matches!(obj.tpe(), SType::SInt | SType::SLong | SType::SBigInt) {
                        if let Some(folded) = fold_numeric_downcast_on_const(&obj, SType::SShort) {
                            folded
                        } else {
                            Downcast::new(obj, SType::SShort)
                                .map_err(|e| {
                                    MirLoweringError::new(format!("{:?}", e), hir_expr.span)
                                })?
                                .into()
                        }
                    } else {
                        Upcast::new(obj, SType::SShort)
                            .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                            .into()
                    }
                }
                // SNumericTypeMethods V6 — property-style (no args). All V3+.
                "toBytes" | "toBits" | "bitwiseInverse" if is_numeric_tpe(&fa.object.tpe) => {
                    let method =
                        lookup_numeric_method(&obj.tpe(), fa.field.as_str()).ok_or_else(|| {
                            MirLoweringError::new(
                                format!("MIR error: numeric method not found: {}", fa.field),
                                hir_expr.span,
                            )
                        })?;
                    PropertyCall::new(obj, method)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "toUnsigned" if matches!(fa.object.tpe, Some(SType::SBigInt)) => {
                    let method =
                        lookup_numeric_method(&obj.tpe(), "toUnsigned").ok_or_else(|| {
                            MirLoweringError::new(
                                "MIR error: BigInt.toUnsigned not found".to_string(),
                                hir_expr.span,
                            )
                        })?;
                    PropertyCall::new(obj, method)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "toSigned" if matches!(fa.object.tpe, Some(SType::SUnsignedBigInt)) => {
                    let method =
                        lookup_numeric_method(&obj.tpe(), "toSigned").ok_or_else(|| {
                            MirLoweringError::new(
                                "MIR error: UnsignedBigInt.toSigned not found".to_string(),
                                hir_expr.span,
                            )
                        })?;
                    PropertyCall::new(obj, method)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "preHeader" => {
                    use ergotree_ir::types::scontext::PRE_HEADER_PROPERTY;
                    PropertyCall::new(obj, PRE_HEADER_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "headers" if matches!(fa.object.tpe, Some(SType::SContext)) => {
                    use ergotree_ir::types::scontext::HEADERS_PROPERTY;
                    PropertyCall::new(obj, HEADERS_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "getEncoded" if matches!(fa.object.tpe, Some(SType::SGroupElement)) => {
                    use ergotree_ir::types::sgroup_elem::GET_ENCODED_METHOD;
                    PropertyCall::new(obj, GET_ENCODED_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                // SHeader properties (15) — V0
                "id" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::ID_PROPERTY;
                    PropertyCall::new(obj, ID_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "version" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::VERSION_PROPERTY;
                    PropertyCall::new(obj, VERSION_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "parentId" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::PARENT_ID_PROPERTY;
                    PropertyCall::new(obj, PARENT_ID_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "ADProofsRoot" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::AD_PROOFS_ROOT_PROPERTY;
                    PropertyCall::new(obj, AD_PROOFS_ROOT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "stateRoot" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::STATE_ROOT_PROPERTY;
                    PropertyCall::new(obj, STATE_ROOT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "transactionsRoot" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::TRANSACTIONS_ROOT_PROPERTY;
                    PropertyCall::new(obj, TRANSACTIONS_ROOT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "timestamp" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::TIMESTAMP_PROPERTY;
                    PropertyCall::new(obj, TIMESTAMP_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "nBits" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::N_BITS_PROPERTY;
                    PropertyCall::new(obj, N_BITS_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "height" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::HEIGHT_PROPERTY;
                    PropertyCall::new(obj, HEIGHT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "extensionRoot" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::EXTENSION_ROOT_PROPERTY;
                    PropertyCall::new(obj, EXTENSION_ROOT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "minerPk" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::MINER_PK_PROPERTY;
                    PropertyCall::new(obj, MINER_PK_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "powOnetimePk" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::POW_ONETIME_PK_PROPERTY;
                    PropertyCall::new(obj, POW_ONETIME_PK_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "powNonce" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::POW_NONCE_PROPERTY;
                    PropertyCall::new(obj, POW_NONCE_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "powDistance" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::POW_DISTANCE_PROPERTY;
                    PropertyCall::new(obj, POW_DISTANCE_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "votes" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::VOTES_PROPERTY;
                    PropertyCall::new(obj, VOTES_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "checkPow" if matches!(fa.object.tpe, Some(SType::SHeader)) => {
                    use ergotree_ir::types::sheader::CHECK_POW_METHOD;
                    PropertyCall::new(obj, CHECK_POW_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                // SPreHeader properties (7) — V0
                "version" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::VERSION_PROPERTY;
                    PropertyCall::new(obj, VERSION_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "parentId" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::PARENT_ID_PROPERTY;
                    PropertyCall::new(obj, PARENT_ID_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "timestamp" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::TIMESTAMP_PROPERTY;
                    PropertyCall::new(obj, TIMESTAMP_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "nBits" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::N_BITS_PROPERTY;
                    PropertyCall::new(obj, N_BITS_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "height" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::HEIGHT_PROPERTY;
                    PropertyCall::new(obj, HEIGHT_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "minerPk" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::MINER_PK_PROPERTY;
                    PropertyCall::new(obj, MINER_PK_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "votes" if matches!(fa.object.tpe, Some(SType::SPreHeader)) => {
                    use ergotree_ir::types::spreheader::VOTES_PROPERTY;
                    PropertyCall::new(obj, VOTES_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "digest" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::DIGEST_METHOD;
                    PropertyCall::new(obj, DIGEST_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "enabledOperations" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::ENABLED_OPERATIONS_METHOD;
                    PropertyCall::new(obj, ENABLED_OPERATIONS_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "keyLength" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::KEY_LENGTH_METHOD;
                    PropertyCall::new(obj, KEY_LENGTH_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "valueLengthOpt" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::VALUE_LENGTH_OPT_METHOD;
                    PropertyCall::new(obj, VALUE_LENGTH_OPT_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "isInsertAllowed" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::IS_INSERT_ALLOWED_METHOD;
                    PropertyCall::new(obj, IS_INSERT_ALLOWED_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "isUpdateAllowed" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::IS_UPDATE_ALLOWED_METHOD;
                    PropertyCall::new(obj, IS_UPDATE_ALLOWED_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "isRemoveAllowed" if matches!(fa.object.tpe, Some(SType::SAvlTree)) => {
                    use ergotree_ir::types::savltree::IS_REMOVE_ALLOWED_METHOD;
                    PropertyCall::new(obj, IS_REMOVE_ALLOWED_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "negate" if matches!(fa.object.tpe, Some(SType::SGroupElement)) => {
                    use ergotree_ir::types::sgroup_elem::NEGATE_METHOD;
                    PropertyCall::new(obj, NEGATE_METHOD.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "LastBlockUtxoRootHash" if matches!(fa.object.tpe, Some(SType::SContext)) => {
                    use ergotree_ir::types::scontext::LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY;
                    PropertyCall::new(obj, LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY.clone())
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                "minerPubKey" if matches!(fa.object.tpe, Some(SType::SContext)) => {
                    // CONTEXT.minerPubKey — same as bare `minerPubKey` global
                    GlobalVars::MinerPubKey.into()
                }
                field
                    if field.len() >= 2
                        && field.starts_with('R')
                        && field[1..].parse::<i8>().is_ok() =>
                {
                    // Register access: .R4, .R5, etc.
                    let reg_id: i8 = field[1..].parse().unwrap();
                    let elem_tpe = fa.type_args.first().cloned().unwrap_or(SType::SAny);
                    let opt_tpe = SType::SOption(elem_tpe.into());
                    ExtractRegisterAs::new(obj, reg_id, opt_tpe)
                        .map_err(|e| MirLoweringError::new(format!("{:?}", e), hir_expr.span))?
                        .into()
                }
                other => {
                    return Err(MirLoweringError::new(
                        format!("MIR error: Unknown field '{}' on type {:?}", other, obj_tpe),
                        hir_expr.span,
                    ))
                }
            }
        }
    };
    let hir_tpe = hir_expr.tpe.clone().ok_or_else(|| {
        MirLoweringError::new(
            format!("MIR error: missing tpe for HIR: {0:?}", hir_expr),
            hir_expr.span,
        )
    })?;
    if mir.tpe() == hir_tpe {
        Ok(mir)
    } else if mir.tpe() == SType::SSigmaProp && hir_tpe == SType::SBoolean {
        // Auto-promotion: &&/|| with a SigmaProp operand produces SigmaProp in MIR
        // even though HIR typed it as SBoolean. This is expected.
        Ok(mir)
    } else if numeric_rank(&mir.tpe()).is_some()
        && numeric_rank(&hir_tpe).is_some()
        && numeric_rank(&mir.tpe()) > numeric_rank(&hir_tpe)
    {
        // Numeric widening: BinOp with mixed numeric types (e.g., Int * Long)
        // produces the wider type in MIR due to implicit upcast, even though
        // HIR typed it using the narrower operand's type.
        Ok(mir)
    } else {
        Err(MirLoweringError::new(
            format!(
                "MIR error: lowered MIR type != HIR type ({0:?} != {1:?})",
                mir.tpe(),
                hir_expr.tpe
            ),
            hir_expr.span,
        ))
    }
}

/// Transform a 2-param fold lambda into a 1-param lambda with tuple destructuring.
/// Fold expects: `(tuple: (AccType, ElemType)) => body` but our compiler produces
/// `(acc: AccType, elem: ElemType) => body`. This rewrites the latter into the former.
fn transform_fold_lambda(fold_op: Expr, span: TextRange) -> Result<Expr, MirLoweringError> {
    match fold_op {
        Expr::FuncValue(fv) if fv.args().len() == 2 => {
            let arg1 = &fv.args()[0];
            let arg2 = &fv.args()[1];
            let acc_id = arg1.idx;
            let acc_tpe = arg1.tpe.clone();
            let elem_id = arg2.idx;
            let elem_tpe = arg2.tpe.clone();

            // Create a new ValId for the single tuple parameter
            let tuple_id = ValId(std::cmp::max(acc_id.0, elem_id.0) + 100);
            let tuple_tpe = SType::STuple(STuple::pair(acc_tpe.clone(), elem_tpe.clone()));

            // Replace ValUse(acc_id) with SelectField(ValUse(tuple_id), _1)
            // Replace ValUse(elem_id) with SelectField(ValUse(tuple_id), _2)
            let new_body = replace_val_uses(
                fv.body().clone(),
                acc_id,
                elem_id,
                tuple_id,
                &tuple_tpe,
                span,
            )?;

            let new_func = FuncValue::new(
                vec![FuncArg {
                    idx: tuple_id,
                    tpe: tuple_tpe,
                }],
                new_body,
            );
            Ok(Expr::FuncValue(new_func))
        }
        other => Ok(other),
    }
}

/// Recursively replace ValUse(acc_id) and ValUse(elem_id) with SelectField on the tuple param.
fn replace_val_uses(
    expr: Expr,
    acc_id: ValId,
    elem_id: ValId,
    tuple_id: ValId,
    tuple_tpe: &SType,
    span: TextRange,
) -> Result<Expr, MirLoweringError> {
    match expr {
        Expr::ValUse(vu) if vu.val_id == acc_id => {
            let tuple_use = Expr::ValUse(ValUse {
                val_id: tuple_id,
                tpe: tuple_tpe.clone(),
            });
            let fi = TupleFieldIndex::try_from(1u8).unwrap();
            Ok(SelectField::new(tuple_use, fi)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::ValUse(vu) if vu.val_id == elem_id => {
            let tuple_use = Expr::ValUse(ValUse {
                val_id: tuple_id,
                tpe: tuple_tpe.clone(),
            });
            let fi = TupleFieldIndex::try_from(2u8).unwrap();
            Ok(SelectField::new(tuple_use, fi)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::BinOp(spanned) => {
            let inner = spanned.expr().clone();
            let new_left =
                replace_val_uses(*inner.left, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_right =
                replace_val_uses(*inner.right, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(BinOp {
                kind: inner.kind,
                left: new_left.into(),
                right: new_right.into(),
            }
            .into())
        }
        Expr::If(if_op) => {
            let new_cond =
                replace_val_uses(*if_op.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_true = replace_val_uses(
                *if_op.true_branch,
                acc_id,
                elem_id,
                tuple_id,
                tuple_tpe,
                span,
            )?;
            let new_false = replace_val_uses(
                *if_op.false_branch,
                acc_id,
                elem_id,
                tuple_id,
                tuple_tpe,
                span,
            )?;
            Ok(If {
                condition: new_cond.into(),
                true_branch: new_true.into(),
                false_branch: new_false.into(),
            }
            .into())
        }
        Expr::BlockValue(spanned) => {
            let inner = spanned.expr().clone();
            let new_items: Result<Vec<Expr>, _> = inner
                .items
                .into_iter()
                .map(|item| replace_val_uses(item, acc_id, elem_id, tuple_id, tuple_tpe, span))
                .collect();
            let new_result =
                replace_val_uses(*inner.result, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(BlockValue {
                items: new_items?,
                result: new_result.into(),
            }
            .into())
        }
        Expr::ValDef(spanned) => {
            let inner = spanned.expr().clone();
            let new_rhs = replace_val_uses(*inner.rhs, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Expr::ValDef(
                ValDef {
                    id: inner.id,
                    rhs: new_rhs.into(),
                }
                .into(),
            ))
        }
        Expr::Tuple(tuple) => {
            let new_items: Result<Vec<Expr>, _> = tuple
                .items
                .as_vec()
                .iter()
                .map(|item| {
                    replace_val_uses(item.clone(), acc_id, elem_id, tuple_id, tuple_tpe, span)
                })
                .collect();
            Ok(Tuple::new(new_items?)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::SelectField(sf) => {
            let inner = sf.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(SelectField::new(new_input, inner.field_index)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::BoolToSigmaProp(bts) => {
            let new_input =
                replace_val_uses(*bts.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(BoolToSigmaProp {
                input: new_input.into(),
            }
            .into())
        }
        Expr::ExtractAmount(ea) => {
            let new_input =
                replace_val_uses(*ea.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(ExtractAmount {
                input: new_input.into(),
            }
            .into())
        }
        Expr::ExtractRegisterAs(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            // elem_tpe is the inner type; ExtractRegisterAs::new expects SOption(elem_tpe)
            let opt_tpe = SType::SOption(inner.elem_tpe.clone());
            Ok(
                ExtractRegisterAs::new(new_input, inner.register_id, opt_tpe)
                    .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                    .into(),
            )
        }
        Expr::OptionGet(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(OptionGet::try_build(new_input)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::SizeOf(so) => {
            let new_input =
                replace_val_uses(*so.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(SizeOf {
                input: new_input.into(),
            }
            .into())
        }
        Expr::PropertyCall(spanned) => {
            let inner = spanned.expr().clone();
            let new_obj = replace_val_uses(*inner.obj, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(PropertyCall::new(new_obj, inner.method)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::ByIndex(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_index =
                replace_val_uses(*inner.index, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(ByIndex::new(new_input, new_index, None)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::Filter(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_cond =
                replace_val_uses(*inner.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Filter::new(new_input, new_cond)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::Exists(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_cond =
                replace_val_uses(*inner.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Exists::new(new_input, new_cond)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::ForAll(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_cond =
                replace_val_uses(*inner.condition, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(ForAll::new(new_input, new_cond)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::FuncValue(fv) => {
            let new_body = replace_val_uses(
                fv.body().clone(),
                acc_id,
                elem_id,
                tuple_id,
                tuple_tpe,
                span,
            )?;
            Ok(Expr::FuncValue(FuncValue::new(
                fv.args().to_vec(),
                new_body,
            )))
        }
        Expr::Append(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            let new_col_2 =
                replace_val_uses(*inner.col_2, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Append::new(new_input, new_col_2)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::LogicalNot(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(LogicalNot::try_build(new_input)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        Expr::Negation(spanned) => {
            let inner = spanned.expr().clone();
            let new_input =
                replace_val_uses(*inner.input, acc_id, elem_id, tuple_id, tuple_tpe, span)?;
            Ok(Negation::try_build(new_input)
                .map_err(|e| MirLoweringError::new(format!("{:?}", e), span))?
                .into())
        }
        // Leaf nodes or nodes that don't contain our ValUse refs
        other => Ok(other),
    }
}

// ---------------------------------------------------------------------------
// Type propagation pass
// ---------------------------------------------------------------------------
//
// After MIR lowering, val annotations may disagree with the actual RHS type.
// For example, `val x: Long = BigInt_expr` produces a ValUse(x, Long) but the
// RHS is BigInt. Downstream `x * y` is then Long*Long (no upcast) instead of
// BigInt*Long (with upcast). The Scala compiler computes types from the graph
// IR, so the annotation is ignored.
//
// This pass walks the MIR tree, collects actual ValDef RHS types, updates
// ValUse types, and re-applies numeric_upcast_pair on BinOps.

/// Propagate actual types from ValDef RHS expressions to ValUse references.
pub fn propagate_val_types(expr: Expr) -> Expr {
    let mut type_map: HashMap<ValId, SType> = HashMap::new();
    propagate_inner(expr, &mut type_map)
}

/// Propagate ValDef SType information through the IR tree. ValDef
/// records its declared type in `type_map`; ValUse looks it up and
/// overrides its own `tpe` to match. Other arms recurse to keep the
/// propagation reaching every ValUse.
///
/// COVERAGE: completeness walker (WS-E.1). Same monotonic-direction
/// reasoning as `cse.rs::replace_all` — a missing arm leaves a ValUse's
/// type un-corrected if it sits inside that node. Adding requires a
/// concrete fixture trace. See
/// [`IR-PASS-COVERAGE-MATRIX.md`](../../tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md).
fn propagate_inner(expr: Expr, type_map: &mut HashMap<ValId, SType>) -> Expr {
    match expr {
        Expr::BlockValue(s) => {
            let inner = s.expr;
            // Process items sequentially so each ValDef's type is available
            // for subsequent items.
            let new_items: Vec<Expr> = inner
                .items
                .into_iter()
                .map(|item| {
                    let item = propagate_inner(item, type_map);
                    // Record actual RHS type for this ValDef
                    if let Expr::ValDef(ref vd_s) = item {
                        let rhs_tpe = vd_s.expr.rhs.tpe();
                        type_map.insert(vd_s.expr.id, rhs_tpe);
                    }
                    item
                })
                .collect();
            let new_result = propagate_inner(*inner.result, type_map);
            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: new_items,
                    result: new_result.into(),
                },
            })
        }
        Expr::ValDef(s) => {
            let new_rhs = propagate_inner(*s.expr.rhs, type_map);
            Expr::ValDef(Spanned {
                source_span: s.source_span,
                expr: ValDef {
                    id: s.expr.id,
                    rhs: new_rhs.into(),
                },
            })
        }
        Expr::ValUse(vu) => {
            if let Some(actual_tpe) = type_map.get(&vu.val_id) {
                if *actual_tpe != vu.tpe {
                    return Expr::ValUse(ValUse {
                        val_id: vu.val_id,
                        tpe: actual_tpe.clone(),
                    });
                }
            }
            Expr::ValUse(vu)
        }
        Expr::BinOp(s) => {
            let inner = s.expr;
            let new_left = propagate_inner(*inner.left, type_map);
            let new_right = propagate_inner(*inner.right, type_map);
            // Re-apply numeric upcast after type propagation
            let (new_left, new_right) = numeric_upcast_pair(new_left, new_right);
            Expr::BinOp(Spanned {
                source_span: s.source_span,
                expr: BinOp {
                    kind: inner.kind,
                    left: new_left.into(),
                    right: new_right.into(),
                },
            })
        }
        Expr::Upcast(uc) => {
            let new_input = propagate_inner(*uc.input, type_map);
            // Remove redundant upcasts (e.g., BigInt → BigInt after propagation)
            if new_input.tpe() == uc.tpe {
                return new_input;
            }
            Expr::Upcast(Upcast {
                input: new_input.into(),
                tpe: uc.tpe,
            })
        }
        Expr::If(if_op) => Expr::If(If {
            condition: propagate_inner(*if_op.condition, type_map).into(),
            true_branch: propagate_inner(*if_op.true_branch, type_map).into(),
            false_branch: propagate_inner(*if_op.false_branch, type_map).into(),
        }),
        Expr::BoolToSigmaProp(bts) => Expr::BoolToSigmaProp(BoolToSigmaProp {
            input: propagate_inner(*bts.input, type_map).into(),
        }),
        Expr::FuncValue(fv) => {
            let new_body = propagate_inner(fv.body().clone(), type_map);
            Expr::FuncValue(FuncValue::new(fv.args().to_vec(), new_body))
        }
        Expr::Filter(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            let cond = propagate_inner(*s.expr.condition, type_map);
            Expr::Filter(Spanned {
                source_span: s.source_span,
                expr: Filter::new(input, cond).expect("Filter in propagate"),
            })
        }
        Expr::Exists(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            let cond = propagate_inner(*s.expr.condition, type_map);
            Expr::Exists(Spanned {
                source_span: s.source_span,
                expr: Exists::new(input, cond).expect("Exists in propagate"),
            })
        }
        Expr::ForAll(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            let cond = propagate_inner(*s.expr.condition, type_map);
            Expr::ForAll(Spanned {
                source_span: s.source_span,
                expr: ForAll::new(input, cond).expect("ForAll in propagate"),
            })
        }
        Expr::Map(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            let mapper = propagate_inner(*s.expr.mapper, type_map);
            Expr::Map(Spanned {
                source_span: s.source_span,
                expr: Map::new(input, mapper).expect("Map in propagate"),
            })
        }
        Expr::Fold(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            let zero = propagate_inner(*s.expr.zero, type_map);
            let fold_op = propagate_inner(*s.expr.fold_op, type_map);
            Expr::Fold(Spanned {
                source_span: s.source_span,
                expr: Fold::new(input, zero, fold_op).expect("Fold in propagate"),
            })
        }
        Expr::ExtractAmount(ea) => Expr::ExtractAmount(ExtractAmount {
            input: propagate_inner(*ea.input, type_map).into(),
        }),
        Expr::ExtractScriptBytes(esb) => Expr::ExtractScriptBytes(ExtractScriptBytes {
            input: propagate_inner(*esb.input, type_map).into(),
        }),
        Expr::ExtractBytes(eb) => Expr::ExtractBytes(ExtractBytes {
            input: propagate_inner(*eb.input, type_map).into(),
        }),
        Expr::ExtractId(ei) => Expr::ExtractId(ExtractId {
            input: propagate_inner(*ei.input, type_map).into(),
        }),
        Expr::ExtractCreationInfo(eci) => Expr::ExtractCreationInfo(ExtractCreationInfo {
            input: propagate_inner(*eci.input, type_map).into(),
        }),
        Expr::ExtractRegisterAs(s) => Expr::ExtractRegisterAs(Spanned {
            source_span: s.source_span,
            expr: ExtractRegisterAs::new(
                propagate_inner(*s.expr.input, type_map),
                s.expr.register_id,
                SType::SOption(s.expr.elem_tpe.clone()),
            )
            .expect("ExtractRegisterAs in propagate"),
        }),
        Expr::SizeOf(so) => Expr::SizeOf(SizeOf {
            input: propagate_inner(*so.input, type_map).into(),
        }),
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: PropertyCall::new(propagate_inner(*s.expr.obj, type_map), s.expr.method)
                .expect("PropertyCall in propagate"),
        }),
        Expr::MethodCall(s) => {
            let obj = propagate_inner(*s.expr.obj, type_map);
            let args: Vec<Expr> = s
                .expr
                .args
                .into_iter()
                .map(|a| propagate_inner(a, type_map))
                .collect();
            Expr::MethodCall(Spanned {
                source_span: s.source_span,
                expr: MethodCall::with_type_args(
                    obj,
                    s.expr.method,
                    args,
                    s.expr.explicit_type_args,
                )
                .expect("MethodCall in propagate"),
            })
        }
        Expr::ByIndex(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            let index = propagate_inner(*s.expr.index, type_map);
            let default = s
                .expr
                .default
                .map(|d| Box::new(propagate_inner(*d, type_map)));
            Expr::ByIndex(Spanned {
                source_span: s.source_span,
                expr: ByIndex::new(input, index, default).expect("ByIndex in propagate"),
            })
        }
        Expr::SelectField(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            Expr::SelectField(Spanned {
                source_span: s.source_span,
                expr: SelectField::new(input, s.expr.field_index)
                    .expect("SelectField in propagate"),
            })
        }
        Expr::OptionGet(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            Expr::OptionGet(Spanned {
                source_span: s.source_span,
                expr: OptionGet::try_build(input).expect("OptionGet in propagate"),
            })
        }
        Expr::OptionIsDefined(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            Expr::OptionIsDefined(Spanned {
                source_span: s.source_span,
                expr: OptionIsDefined::try_build(input).expect("OptionIsDefined in propagate"),
            })
        }
        Expr::LogicalNot(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            Expr::LogicalNot(Spanned {
                source_span: s.source_span,
                expr: LogicalNot::try_build(input).expect("LogicalNot in propagate"),
            })
        }
        Expr::Negation(s) => {
            let input = propagate_inner(*s.expr.input, type_map);
            Expr::Negation(Spanned {
                source_span: s.source_span,
                expr: Negation::try_build(input).expect("Negation in propagate"),
            })
        }
        Expr::SigmaPropBytes(spb) => Expr::SigmaPropBytes(SigmaPropBytes {
            input: propagate_inner(*spb.input, type_map).into(),
        }),
        Expr::CalcBlake2b256(cb) => Expr::CalcBlake2b256(CalcBlake2b256 {
            input: propagate_inner(*cb.input, type_map).into(),
        }),
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa
                .items
                .into_iter()
                .map(|i| propagate_inner(i, type_map))
                .collect();
            Expr::SigmaAnd(SigmaAnd {
                items: items.try_into().expect("SigmaAnd in propagate"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so
                .items
                .into_iter()
                .map(|i| propagate_inner(i, type_map))
                .collect();
            Expr::SigmaOr(SigmaOr {
                items: items.try_into().expect("SigmaOr in propagate"),
            })
        }
        Expr::Tuple(t) => {
            let items: Vec<Expr> = t
                .items
                .into_iter()
                .map(|i| propagate_inner(i, type_map))
                .collect();
            Expr::Tuple(Tuple::new(items).expect("Tuple in propagate"))
        }
        Expr::Collection(c) => match c {
            Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> = items
                    .into_iter()
                    .map(|i| propagate_inner(i, type_map))
                    .collect();
                Expr::Collection(Collection::Exprs {
                    elem_tpe,
                    items: new_items,
                })
            }
            other => Expr::Collection(other),
        },
        Expr::And(a) => Expr::And(Spanned {
            source_span: a.source_span,
            expr: ergotree_ir::mir::and::And {
                input: propagate_inner(*a.expr.input, type_map).into(),
            },
        }),
        Expr::Or(o) => Expr::Or(Spanned {
            source_span: o.source_span,
            expr: ergotree_ir::mir::or::Or {
                input: propagate_inner(*o.expr.input, type_map).into(),
            },
        }),
        Expr::Downcast(dc) => {
            let input = propagate_inner(*dc.input, type_map);
            Expr::Downcast(Downcast::new(input, dc.tpe).expect("Downcast in propagate"))
        }
        Expr::Slice(s) => Expr::Slice(Spanned {
            source_span: s.source_span,
            expr: Slice::new(
                propagate_inner(*s.expr.input, type_map),
                propagate_inner(*s.expr.from, type_map),
                propagate_inner(*s.expr.until, type_map),
            )
            .expect("Slice in propagate"),
        }),
        Expr::TreeLookup(s) => Expr::TreeLookup(Spanned {
            source_span: s.source_span,
            expr: TreeLookup {
                tree: propagate_inner(*s.expr.tree, type_map).into(),
                key: propagate_inner(*s.expr.key, type_map).into(),
                proof: propagate_inner(*s.expr.proof, type_map).into(),
            },
        }),
        Expr::CreateProveDlog(cpd) => Expr::CreateProveDlog(CreateProveDlog {
            input: propagate_inner(*cpd.input, type_map).into(),
        }),
        Expr::LongToByteArray(ltba) => Expr::LongToByteArray(LongToByteArray {
            input: propagate_inner(*ltba.input, type_map).into(),
        }),
        Expr::Apply(app) => {
            let func = propagate_inner(*app.func, type_map);
            let args: Vec<Expr> = app
                .args
                .into_iter()
                .map(|a| propagate_inner(a, type_map))
                .collect();
            ergotree_ir::mir::apply::Apply::new(func, args)
                .map(Expr::Apply)
                .expect("Apply in propagate")
        }
        Expr::Atleast(s) => {
            let bound = propagate_inner(*s.bound, type_map);
            let input = propagate_inner(*s.input, type_map);
            ergotree_ir::mir::atleast::Atleast::new(bound, input)
                .map(Expr::Atleast)
                .expect("Atleast in propagate")
        }
        // Leaf nodes: Const, GlobalVars, Context, GetVar, etc.
        other => other,
    }
}

impl From<hir::BinaryOp> for BinOpKind {
    fn from(op: hir::BinaryOp) -> Self {
        match op {
            BinaryOp::Plus => ArithOp::Plus.into(),
            BinaryOp::Minus => ArithOp::Minus.into(),
            BinaryOp::Multiply => ArithOp::Multiply.into(),
            BinaryOp::Divide => ArithOp::Divide.into(),
            BinaryOp::Modulo => ArithOp::Modulo.into(),
            BinaryOp::Eq => RelationOp::Eq.into(),
            BinaryOp::Neq => RelationOp::NEq.into(),
            BinaryOp::Gt => RelationOp::Gt.into(),
            BinaryOp::Lt => RelationOp::Lt.into(),
            BinaryOp::Ge => RelationOp::Ge.into(),
            BinaryOp::Le => RelationOp::Le.into(),
            BinaryOp::And => LogicalOp::And.into(),
            BinaryOp::Or => LogicalOp::Or.into(),
            BinaryOp::BitAnd => BitOp::BitAnd.into(),
            BinaryOp::BitOr => BitOp::BitOr.into(),
            BinaryOp::BitXor => BitOp::BitXor.into(),
            BinaryOp::Shl => BitOp::BitShiftLeft.into(),
            BinaryOp::Shr => BitOp::BitShiftRight.into(),
            BinaryOp::UShr => BitOp::BitShiftRightZeroed.into(),
            // ConcatColl is desugared into Append before reaching this conversion;
            // see the Binary lowering arm above. Reaching here means the desugar
            // was skipped — that's a bug, not a real BinOpKind.
            BinaryOp::ConcatColl => {
                unreachable!("BinaryOp::ConcatColl should be lowered to Append, not BinOpKind")
            }
        }
    }
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = crate::parser::parse(input);
    let syntax = parse.syntax();
    let root = crate::ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root).unwrap();
    let binder = crate::binder::Binder::new(crate::script_env::ScriptEnv::new());
    let bind = binder.bind(hir).unwrap();
    let typed = crate::type_infer::assign_type(bind).unwrap();
    let res = lower(typed).unwrap();
    expected_tree.assert_eq(&res.debug_tree());
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: GlobalVars(
                                Height,
                            ),
                            right: GlobalVars(
                                Height,
                            ),
                        },
                    },
                )"#]],
        )
    }

    #[test]
    fn literal_int() {
        check(
            "42",
            expect![[r#"
                Const(
                    "42: SInt",
                )"#]],
        );
    }

    #[test]
    fn literal_long() {
        check(
            "42L",
            expect![[r#"
                Const(
                    "42: SLong",
                )"#]],
        );
    }

    #[test]
    fn bin_numeric_int() {
        // Const+Const arithmetic folds at MIR-lower time to mirror Scala's
        // graph-IR `propagateBinOp`. See `fold_arith_const_const`.
        check(
            "4+2",
            expect![[r#"
                Const(
                    "6: SInt",
                )"#]],
        );
    }

    #[test]
    fn bin_numeric_long() {
        check(
            "4L+2L",
            expect![[r#"
                Const(
                    "6: SLong",
                )"#]],
        );
    }

    #[test]
    fn literal_bool_true() {
        check(
            "true",
            expect![[r#"
                Const(
                    "true: SBoolean",
                )"#]],
        );
    }

    #[test]
    fn literal_bool_false() {
        check(
            "false",
            expect![[r#"
                Const(
                    "false: SBoolean",
                )"#]],
        );
    }

    #[test]
    fn comparison_gt() {
        check(
            "HEIGHT > 0",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Relation(
                                Gt,
                            ),
                            left: GlobalVars(
                                Height,
                            ),
                            right: Const(
                                "0: SInt",
                            ),
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn sigmaprop_bool() {
        check(
            "sigmaProp(true)",
            expect![[r#"
                BoolToSigmaProp(
                    BoolToSigmaProp {
                        input: Const(
                            "true: SBoolean",
                        ),
                    },
                )"#]],
        );
    }
}
