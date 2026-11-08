//! Per-type equality comparison with JIT costing.
//!
//! Mirrors Scala sigmastate-interpreter's `DataValueComparer`: every equality
//! check charges a type-specific cost before (or instead of) structural
//! comparison. Constants are drawn from the Scala reference and validated by
//! PR 846 (arkadianet/jit-costing) against 19,549 mainnet transactions.

use ergotree_ir::chain::context::Context;
use ergotree_ir::mir::value::{CollKind, NativeColl, Value};
use ergotree_ir::sigma_protocol::sigma_boolean::{
    SigmaBoolean, SigmaConjecture, SigmaConjectureItems, SigmaProofOfKnowledgeTree,
};
use ergotree_ir::types::stype::SType;

use super::EvalError;

// --- Fixed per-type equality costs (JitCost units) ---
const EQ_PRIM_COST: u64 = 3;
const EQ_BIGINT_COST: u64 = 5;
const EQ_GROUP_ELEMENT_COST: u64 = 172;
const EQ_TUPLE_COST: u64 = 4;
const EQ_OPTION_COST: u64 = 4;
const EQ_AVL_TREE_COST: u64 = 6;
const EQ_BOX_COST: u64 = 6;
const EQ_PREHEADER_COST: u64 = 4;
const EQ_HEADER_COST: u64 = 6;

// Scala's MatchType dispatch cost (`CostOf_MatchType` = 1), charged on each
// type-match step: the collection dispatch (before its length-mismatch
// short-circuit, so it is always paid), the SigmaProp dispatch, and once per
// SigmaBoolean tree node.
const MATCH_TYPE_COST: u64 = 1;

// Per-element collection equality costs as (base, per_chunk, chunk_size),
// matching `Context::add_per_item_jit_cost`'s argument shape.
const EQ_COLL_BYTE_PER_ITEM: (u32, u32, u32) = (15, 2, 128);
const EQ_COLL_SHORT_PER_ITEM: (u32, u32, u32) = (15, 2, 96);
const EQ_COLL_INT_PER_ITEM: (u32, u32, u32) = (15, 2, 64);
const EQ_COLL_LONG_PER_ITEM: (u32, u32, u32) = (15, 2, 48);
const EQ_COLL_BOOLEAN_PER_ITEM: (u32, u32, u32) = (15, 2, 128);
const EQ_COLL_BIGINT_PER_ITEM: (u32, u32, u32) = (15, 7, 5);
const EQ_COLL_GROUP_ELEMENT_PER_ITEM: (u32, u32, u32) = (15, 5, 1);
const EQ_COLL_AVL_TREE_PER_ITEM: (u32, u32, u32) = (15, 5, 2);
const EQ_COLL_BOX_PER_ITEM: (u32, u32, u32) = (15, 5, 1);
const EQ_COLL_PREHEADER_PER_ITEM: (u32, u32, u32) = (15, 3, 1);
const EQ_COLL_HEADER_PER_ITEM: (u32, u32, u32) = (15, 5, 1);
const EQ_COLL_DEFAULT_PER_ITEM: (u32, u32, u32) = (10, 2, 1);

/// Compare two Values for equality, charging per-type costs to `ctx`.
/// Returns `Ok(true)` / `Ok(false)`, or an `EvalError::CostError` if the
/// cumulative JIT cost limit is exceeded during charging.
pub(crate) fn eq_with_cost<'ctx>(
    lv: &Value<'ctx>,
    rv: &Value<'ctx>,
    ctx: &Context<'ctx>,
) -> Result<bool, EvalError> {
    match (lv, rv) {
        (Value::Boolean(_), Value::Boolean(_))
        | (Value::Byte(_), Value::Byte(_))
        | (Value::Short(_), Value::Short(_))
        | (Value::Int(_), Value::Int(_))
        | (Value::Long(_), Value::Long(_)) => {
            ctx.add_jit_cost(EQ_PRIM_COST)?;
            Ok(lv == rv)
        }

        (Value::BigInt(_), Value::BigInt(_))
        | (Value::UnsignedBigInt(_), Value::UnsignedBigInt(_)) => {
            ctx.add_jit_cost(EQ_BIGINT_COST)?;
            Ok(lv == rv)
        }

        (Value::GroupElement(_), Value::GroupElement(_)) => {
            ctx.add_jit_cost(EQ_GROUP_ELEMENT_COST)?;
            Ok(lv == rv)
        }

        (Value::CBox(_), Value::CBox(_)) => {
            ctx.add_jit_cost(EQ_BOX_COST)?;
            Ok(lv == rv)
        }

        (Value::AvlTree(_), Value::AvlTree(_)) => {
            ctx.add_jit_cost(EQ_AVL_TREE_COST)?;
            Ok(lv == rv)
        }

        (Value::Tup(l_items), Value::Tup(r_items)) => {
            ctx.add_jit_cost(EQ_TUPLE_COST)?;
            if l_items.len() != r_items.len() {
                return Ok(false);
            }
            for (l, r) in l_items.iter().zip(r_items.iter()) {
                if !eq_with_cost(l, r, ctx)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }

        (Value::Opt(l_opt), Value::Opt(r_opt)) => {
            ctx.add_jit_cost(EQ_OPTION_COST)?;
            match (l_opt.as_deref(), r_opt.as_deref()) {
                (None, None) => Ok(true),
                (Some(l), Some(r)) => eq_with_cost(l, r, ctx),
                _ => Ok(false),
            }
        }

        (Value::Coll(l_coll), Value::Coll(r_coll)) => {
            // MatchType dispatch cost always paid, matching Scala's
            // DataValueComparer case 2 (bug 4).
            ctx.add_jit_cost(MATCH_TYPE_COST)?;
            let n = l_coll.len();
            if n != r_coll.len() {
                // Scala short-circuits on length mismatch without charging
                // per-item or base cost (bug 5).
                return Ok(false);
            }
            // Both JVM coll-eq paths short-circuit the *per-item cost* on the
            // first mismatch. `addSeqCost(costKind){ block }` charges
            // `costKind.cost(i)`, where the block returns `i` = the number of
            // items actually compared (CErgoTreeEvaluator.scala:412;
            // `equalCOA_Prim`/`equalColls` both loop `while (i < len && okEqual)`
            // and return `i`). So the per-item charge is on the *compared count*
            // (first-mismatch index + 1, or the full length when equal), NOT the
            // full operand length. (h28931 colleq cost divergence.)
            match coll_eq_cost(l_coll) {
                Some((base, per_chunk, chunk_size)) => {
                    // COA leaf colls: JVM `equalCOA_*` bulk-compares and returns
                    // the compared count. Derive it from the first mismatch; the
                    // result stays the proven `lv == rv`.
                    let compared = coll_eq_compared_count(l_coll, r_coll);
                    ctx.add_per_item_jit_cost(base, per_chunk, chunk_size, compared)?;
                    Ok(lv == rv)
                }
                None => {
                    // Composite element colls: JVM generic `equalColls` recurses
                    // per element (nested costs accrue during the loop) and
                    // charges `EQ_Coll` on the compared count *after* the loop.
                    let (base, per_chunk, chunk_size) = EQ_COLL_DEFAULT_PER_ITEM;
                    let l_items = l_coll.as_vec();
                    let r_items = r_coll.as_vec();
                    let mut compared: u32 = 0;
                    let mut all_equal = true;
                    for (l, r) in l_items.iter().zip(r_items.iter()) {
                        compared += 1;
                        if !eq_with_cost(l, r, ctx)? {
                            all_equal = false;
                            break;
                        }
                    }
                    ctx.add_per_item_jit_cost(base, per_chunk, chunk_size, compared)?;
                    Ok(all_equal)
                }
            }
        }

        (Value::Header(_), Value::Header(_)) => {
            ctx.add_jit_cost(EQ_HEADER_COST)?;
            Ok(lv == rv)
        }

        (Value::PreHeader(_), Value::PreHeader(_)) => {
            ctx.add_jit_cost(EQ_PREHEADER_COST)?;
            Ok(lv == rv)
        }

        (Value::SigmaProp(l), Value::SigmaProp(r)) => {
            // Scala `DataValueComparer.equalDataValues` SigmaProp case
            // (DataValueComparer.scala:353): one MatchType for the dispatch,
            // then `equalSigmaBoolean` walks both trees.
            ctx.add_jit_cost(MATCH_TYPE_COST)?;
            eq_sigma_bool_with_cost(l.value(), r.value(), ctx)
        }

        // String, Unit, Lambda, Context, Global, and any cross-type comparisons
        // (which PartialEq returns false for anyway).
        _ => {
            ctx.add_jit_cost(EQ_PRIM_COST)?;
            Ok(lv == rv)
        }
    }
}

/// Compare two `SigmaBoolean` trees, charging cost per Scala `equalSigmaBoolean`
/// (DataValueComparer.scala:253): `MATCH_TYPE_COST` once per node + the
/// `EQ_GROUP_ELEMENT_COST` per EcPoint. Short-circuits exactly as Scala's `&&`
/// and length checks do, so an unequal pair is charged only up to the first
/// mismatch.
fn eq_sigma_bool_with_cost(
    l: &SigmaBoolean,
    r: &SigmaBoolean,
    ctx: &Context<'_>,
) -> Result<bool, EvalError> {
    use SigmaBoolean::{ProofOfKnowledge, SigmaConjecture as Conj, TrivialProp};
    use SigmaConjecture::{Cand, Cor, Cthreshold};
    use SigmaProofOfKnowledgeTree::{ProveDhTuple, ProveDlog};
    ctx.add_jit_cost(MATCH_TYPE_COST)?; // once per node
    match (l, r) {
        (ProofOfKnowledge(ProveDlog(x)), ProofOfKnowledge(ProveDlog(y))) => {
            ctx.add_jit_cost(EQ_GROUP_ELEMENT_COST)?;
            Ok(x.h == y.h)
        }
        (ProofOfKnowledge(ProveDhTuple(x)), ProofOfKnowledge(ProveDhTuple(y))) => {
            // Four `equalECPoint`s, &&-short-circuited: each one reached charges
            // EQ_GROUP_ELEMENT_COST (including the mismatching one); a mismatch
            // stops the rest.
            for (lp, rp) in [(&x.g, &y.g), (&x.h, &y.h), (&x.u, &y.u), (&x.v, &y.v)] {
                ctx.add_jit_cost(EQ_GROUP_ELEMENT_COST)?;
                if lp != rp {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (TrivialProp(a), TrivialProp(b)) => Ok(a == b),
        (Conj(Cand(x)), Conj(Cand(y))) => eq_sigma_bools_with_cost(&x.items, &y.items, ctx),
        (Conj(Cor(x)), Conj(Cor(y))) => eq_sigma_bools_with_cost(&x.items, &y.items, ctx),
        (Conj(Cthreshold(x)), Conj(Cthreshold(y))) => {
            Ok(x.k == y.k && eq_sigma_bools_with_cost(&x.children, &y.children, ctx)?)
        }
        // Mismatched node types: the node's MatchType is charged above; the
        // comparison is false (Scala's `case _ => false`).
        _ => Ok(false),
    }
}

/// `equalSigmaBooleans` (DataValueComparer.scala:241): length check (no per-item
/// charge on mismatch), then per-child `eq_sigma_bool_with_cost`, short-circuiting.
fn eq_sigma_bools_with_cost(
    xs: &SigmaConjectureItems<SigmaBoolean>,
    ys: &SigmaConjectureItems<SigmaBoolean>,
    ctx: &Context<'_>,
) -> Result<bool, EvalError> {
    if xs.len() != ys.len() {
        return Ok(false);
    }
    for (x, y) in xs.iter().zip(ys.iter()) {
        if !eq_sigma_bool_with_cost(x, y, ctx)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Per-item equality cost for a collection (keyed on element type) **and**
/// whether the collection is a COA (CollOverArray) leaf — the single source of
/// truth for the leaf-vs-composite partition. `Some((base, per_chunk,
/// chunk_size))` = COA leaf, which JVM `equalColls_Dispatch` bulk-compares
/// without recursion. `None` = a composite element type (Coll/Tuple/Option/
/// SigmaProp/...), which pays `EQ_COLL_DEFAULT_PER_ITEM` and recurses per
/// element via JVM's generic `equalColls`. Mirrors Scala's per-type array cost
/// kinds.
fn coll_eq_cost(coll: &CollKind<Value<'_>>) -> Option<(u32, u32, u32)> {
    match coll {
        CollKind::NativeColl(NativeColl::CollByte(_)) => Some(EQ_COLL_BYTE_PER_ITEM),
        CollKind::WrappedColl { elem_tpe, .. } => match elem_tpe {
            SType::SShort => Some(EQ_COLL_SHORT_PER_ITEM),
            SType::SInt => Some(EQ_COLL_INT_PER_ITEM),
            SType::SLong => Some(EQ_COLL_LONG_PER_ITEM),
            SType::SBoolean => Some(EQ_COLL_BOOLEAN_PER_ITEM),
            SType::SBigInt | SType::SUnsignedBigInt => Some(EQ_COLL_BIGINT_PER_ITEM),
            SType::SGroupElement => Some(EQ_COLL_GROUP_ELEMENT_PER_ITEM),
            SType::SAvlTree => Some(EQ_COLL_AVL_TREE_PER_ITEM),
            SType::SBox => Some(EQ_COLL_BOX_PER_ITEM),
            SType::SPreHeader => Some(EQ_COLL_PREHEADER_PER_ITEM),
            SType::SHeader => Some(EQ_COLL_HEADER_PER_ITEM),
            _ => None,
        },
    }
}

/// JVM coll-eq per-item cost short-circuits on the first mismatch: the count it
/// charges is the number of items examined before the result is decided —
/// `first_mismatch_index + 1`, or the full length when the operands are equal
/// (`equalCOA_Prim`/`equalColls` return this `i`). Returns that count for a COA
/// (leaf) collection pair already known to have equal length. Mixed collection
/// kinds cannot occur for a COA element type (`Coll[Byte]` is always native,
/// every other leaf type always wrapped with a matching element type); the `n`
/// fallback keeps the prior full-length charge for that unreachable shape.
fn coll_eq_compared_count(l: &CollKind<Value<'_>>, r: &CollKind<Value<'_>>) -> u32 {
    let first_mismatch = match (l, r) {
        (
            CollKind::NativeColl(NativeColl::CollByte(la)),
            CollKind::NativeColl(NativeColl::CollByte(ra)),
        ) => la.iter().zip(ra.iter()).position(|(a, b)| a != b),
        (CollKind::WrappedColl { items: li, .. }, CollKind::WrappedColl { items: ri, .. }) => {
            li.iter().zip(ri.iter()).position(|(a, b)| a != b)
        }
        _ => None,
    };
    match first_mismatch {
        Some(idx) => idx as u32 + 1,
        None => l.len() as u32,
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use alloc::vec;
    use sigma_test_util::force_any_val;

    #[test]
    fn primitive_eq_charges_prim_cost() {
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let lv: Value<'_> = Value::Int(42);
        let rv: Value<'_> = Value::Int(42);
        assert!(eq_with_cost(&lv, &rv, &ctx).unwrap());
        assert_eq!(ctx.jit_cost_value() - before, EQ_PRIM_COST);
    }

    #[test]
    fn coll_eq_charges_match_type_plus_per_item() {
        // Bug 4 regression: equal Coll[Int] of length 3 must pay MatchType(1)
        // + per-item SInt cost (base=15, per_chunk=2, chunk_size=64 → 1 chunk
        // of 3 items = 15 + 2 = 17). Total = 1 + 17 = 18.
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let items: Arc<[Value<'_>]> = Arc::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let lv: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: items.clone(),
        });
        let rv: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items,
        });
        assert!(eq_with_cost(&lv, &rv, &ctx).unwrap());
        assert_eq!(ctx.jit_cost_value() - before, 18);
    }

    #[test]
    fn nested_coll_eq_recurses_element_cost() {
        // B4: a Coll with a composite element type (here Coll[Coll[Int]]) must
        // recurse eq_with_cost per element -- charging the nested MatchType +
        // per-item -- matching JVM's generic `equalColls`; leaf-element colls
        // (the inner Coll[Int]) still bulk-compare. For [[1,2,3],[4,5,6]] vs
        // itself: outer = MatchType 1 + default per-item (10,2,1) over 2 =
        // 1+14 = 15; each inner Coll[Int] len 3 = MatchType 1 + (15,2,64) over
        // 3 = 1+17 = 18; total = 15 + 18 + 18 = 51 (pre-fix, no recursion: 15).
        let ctx = force_any_val::<Context>();
        let inner_a: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: Arc::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        });
        let inner_b: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: Arc::from(vec![Value::Int(4), Value::Int(5), Value::Int(6)]),
        });
        let outer: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SColl(Arc::new(SType::SInt)),
            items: Arc::from(vec![inner_a, inner_b]),
        });
        let before = ctx.jit_cost_value();
        assert!(eq_with_cost(&outer, &outer.clone(), &ctx).unwrap());
        assert_eq!(ctx.jit_cost_value() - before, 51);
    }

    #[test]
    fn coll_eq_length_mismatch_short_circuits() {
        // Bug 5 regression: Coll[Int] of different lengths must pay only the
        // MatchType(1) dispatch cost, NOT per-item cost (early false return).
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let items_a: Arc<[Value<'_>]> = Arc::from(vec![Value::Int(1), Value::Int(2)]);
        let items_b: Arc<[Value<'_>]> = Arc::from(vec![Value::Int(1)]);
        let lv: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: items_a,
        });
        let rv: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: items_b,
        });
        assert!(!eq_with_cost(&lv, &rv, &ctx).unwrap());
        assert_eq!(ctx.jit_cost_value() - before, MATCH_TYPE_COST);
    }

    /// Charge for equality of two equal empty `Coll[elem_tpe]` (wrapped form):
    /// MatchType dispatch + per-item cost at n=0.
    fn empty_wrapped_coll_eq_cost(elem_tpe: SType) -> u64 {
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let empty: Arc<[Value<'_>]> = Arc::from(Vec::<Value<'_>>::new());
        let lv: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: elem_tpe.clone(),
            items: empty.clone(),
        });
        let rv: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe,
            items: empty,
        });
        assert!(eq_with_cost(&lv, &rv, &ctx).unwrap());
        ctx.jit_cost_value() - before
    }

    #[test]
    fn empty_coll_eq_cs_ge_2_charges_one_chunk() {
        // n=0 regression (the mainnet 1,520,814 finding): for chunkSize>=2
        // element types an empty Coll must still pay one chunk, mirroring Scala
        // PerItemCost.chunks(0) = (0-1)/cs + 1 = 1 (signed, toward-zero div).
        // Cost = MatchType(1) + base(15) + 1*per_chunk. Before the fix Rust's
        // ceiling chunks(0)=0 charged only base -> undercharge vs the JVM.
        // Long(cs=48) / Int(cs=64) / Short(cs=96) / Boolean(cs=128): per_chunk=2.
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SLong), 18);
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SInt), 18);
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SShort), 18);
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SBoolean), 18);
        // BigInt: per_chunk=7, cs=5 -> 1 + 15 + 7 = 23.
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SBigInt), 23);
        // AvlTree: per_chunk=5, cs=2 -> 1 + 15 + 5 = 21.
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SAvlTree), 21);
    }

    #[test]
    fn empty_coll_byte_eq_charges_one_chunk() {
        // Coll[Byte] is a NativeColl (cs=128, per_chunk=2). Empty -> one chunk:
        // MatchType(1) + base(15) + 2 = 18.
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let empty_bytes: Arc<[i8]> = Arc::from(Vec::<i8>::new());
        let lv: Value<'_> = Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
            empty_bytes.clone(),
        )));
        let rv: Value<'_> = Value::Coll(CollKind::NativeColl(NativeColl::CollByte(empty_bytes)));
        assert!(eq_with_cost(&lv, &rv, &ctx).unwrap());
        assert_eq!(ctx.jit_cost_value() - before, 18);
    }

    #[test]
    fn empty_coll_cs_eq_1_charges_base_only() {
        // chunkSize==1 types stay at base only: Scala chunks(0) = (0-1)/1 + 1 = 0.
        // Cost = MatchType(1) + base(15) + 0 = 16. These already matched the JVM
        // before the fix; assert the n=0 change does NOT move them.
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SBox), 16);
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SGroupElement), 16);
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SHeader), 16);
        assert_eq!(empty_wrapped_coll_eq_cost(SType::SPreHeader), 16);
    }

    #[test]
    fn nonempty_coll_long_path_byte_identical() {
        // Prove n>=1 is byte-identical to the old ceiling formula across a chunk
        // boundary. Long: base=15, per_chunk=2, cs=48. Total = MatchType(1) +
        // base + chunks(n)*per_chunk, where chunks(n) == ceil(n/48) for n>=1.
        //   len 1  -> chunks=1 -> 1 + 15 + 2  = 18
        //   len 48 -> chunks=1 -> 1 + 15 + 2  = 18  (last item still in chunk 1)
        //   len 49 -> chunks=2 -> 1 + 15 + 4  = 20  (spills into chunk 2)
        let cost = |len: usize| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let items: Arc<[Value<'_>]> =
                Arc::from((0..len as i64).map(Value::Long).collect::<Vec<Value<'_>>>());
            let lv: Value<'_> = Value::Coll(CollKind::WrappedColl {
                elem_tpe: SType::SLong,
                items: items.clone(),
            });
            let rv: Value<'_> = Value::Coll(CollKind::WrappedColl {
                elem_tpe: SType::SLong,
                items,
            });
            assert!(eq_with_cost(&lv, &rv, &ctx).unwrap());
            ctx.jit_cost_value() - before
        };
        assert_eq!(cost(1), 18);
        assert_eq!(cost(48), 18);
        assert_eq!(cost(49), 20);
    }

    /// CONSENSUS PARITY: `SigmaProp == SigmaProp` matches Scala
    /// `DataValueComparer.equalDataValues`'s dedicated `SigmaProp` case
    /// (DataValueComparer.scala:353) → `equalSigmaBoolean`: `MatchType(1)` for the
    /// dispatch + `MatchType(1)` per tree node + `EQ_GroupElement(172)` per EcPoint.
    ///   ProveDlog    == ProveDlog:    1 + 1 + 172     = 174
    ///   ProveDHTuple == ProveDHTuple: 1 + 1 + 4 * 172 = 690
    #[test]
    fn sigmaprop_eq_matches_scala_equalsigmaboolean() {
        use ergotree_ir::sigma_protocol::sigma_boolean::{
            ProveDhTuple, ProveDlog, SigmaBoolean, SigmaProp,
        };
        let cost_of = |sp: SigmaProp| -> u64 {
            let ctx = force_any_val::<Context>();
            let v: Value<'_> = Value::sigma_prop(sp);
            let rv = v.clone();
            let before = ctx.jit_cost_value();
            assert!(eq_with_cost(&v, &rv, &ctx).unwrap());
            ctx.jit_cost_value() - before
        };
        let dlog = SigmaProp::new(SigmaBoolean::from(force_any_val::<ProveDlog>()));
        let dht = SigmaProp::new(SigmaBoolean::from(force_any_val::<ProveDhTuple>()));
        assert_eq!(
            cost_of(dlog),
            174,
            "ProveDlog==ProveDlog: Scala charges MatchType*2 + EQ_GroupElement(172)"
        );
        assert_eq!(
            cost_of(dht),
            690,
            "ProveDHTuple==ProveDHTuple: Scala charges MatchType*2 + 4*EQ_GroupElement(172)"
        );
    }

    #[test]
    fn coll_eq_coa_early_mismatch_charges_compared_count() {
        // h28931 class: an equal-length COA eq that mismatches early charges the
        // per-item cost on the *compared count* (first-mismatch index + 1), not
        // the full length. Coll[Long]: base=15, per_chunk=2, cs=48.
        //   mismatch @ idx 0  -> compared 1  -> chunks(1)=1  -> 1 + 15 + 2 = 18
        //   mismatch @ idx 47 -> compared 48 -> chunks(48)=1 -> 1 + 15 + 2 = 18
        //   mismatch @ idx 48 -> compared 49 -> chunks(49)=2 -> 1 + 15 + 4 = 20
        // Pre-fix every case charged chunks(full len)=chunks(49)=2 -> 20, so the
        // early-mismatch cases over-charged by 2.
        let cost = |len: usize, mismatch_at: usize| -> (bool, u64) {
            let ctx = force_any_val::<Context>();
            let l: Arc<[Value<'_>]> =
                Arc::from((0..len as i64).map(Value::Long).collect::<Vec<Value<'_>>>());
            let mut r_vec: Vec<Value<'_>> = (0..len as i64).map(Value::Long).collect();
            r_vec[mismatch_at] = Value::Long(-1); // -1 is never equal to its index
            let r: Arc<[Value<'_>]> = Arc::from(r_vec);
            let lv = Value::Coll(CollKind::WrappedColl {
                elem_tpe: SType::SLong,
                items: l,
            });
            let rv = Value::Coll(CollKind::WrappedColl {
                elem_tpe: SType::SLong,
                items: r,
            });
            let before = ctx.jit_cost_value();
            let eq = eq_with_cost(&lv, &rv, &ctx).unwrap();
            (eq, ctx.jit_cost_value() - before)
        };
        assert_eq!(cost(49, 0), (false, 18));
        assert_eq!(cost(49, 47), (false, 18));
        assert_eq!(cost(49, 48), (false, 20)); // last item: compared == full length
    }

    #[test]
    fn coll_eq_generic_early_mismatch_charges_compared_count() {
        // Composite-element coll: the generic `EQ_Coll` per-item cost is charged
        // on the compared count *after* recursing, and recursion short-circuits
        // at the first unequal element. Outer Coll[Coll[Int]] of length 3 whose
        // element 0 already differs (elements 1 and 2 are never reached):
        //   outer MatchType                                            = 1
        //   recurse element 0: inner Coll[Int] [1,2,3] vs [9,2,3],
        //     mismatch @ idx 0 -> compared 1 -> 15+2: MatchType 1 + 17 = 18
        //   outer EQ_Coll on compared count 1 (base 10, per_chunk 2, cs 1):
        //     chunks(1)=1 -> 10 + 2                                    = 12
        //   total                                                      = 31
        // Pre-fix charged EQ_Coll on the full length 3 (=16) BEFORE recursing,
        // for a total of 35.
        let ctx = force_any_val::<Context>();
        let inner_l: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: Arc::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        });
        let inner_r: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: Arc::from(vec![Value::Int(9), Value::Int(2), Value::Int(3)]),
        });
        let other: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SInt,
            items: Arc::from(vec![Value::Int(7), Value::Int(8), Value::Int(9)]),
        });
        let outer_l: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SColl(Arc::new(SType::SInt)),
            items: Arc::from(vec![inner_l, other.clone(), other.clone()]),
        });
        let outer_r: Value<'_> = Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SColl(Arc::new(SType::SInt)),
            items: Arc::from(vec![inner_r, other.clone(), other]),
        });
        let before = ctx.jit_cost_value();
        assert!(!eq_with_cost(&outer_l, &outer_r, &ctx).unwrap());
        assert_eq!(ctx.jit_cost_value() - before, 31);
    }
}
