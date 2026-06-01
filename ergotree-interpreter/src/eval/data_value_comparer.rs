//! Per-type equality comparison with JIT costing.
//!
//! Mirrors Scala sigmastate-interpreter's `DataValueComparer`: every equality
//! check charges a type-specific cost before (or instead of) structural
//! comparison. Constants are drawn from the Scala reference and validated by
//! PR 846 (arkadianet/jit-costing) against 19,549 mainnet transactions.

use ergotree_ir::chain::context::Context;
use ergotree_ir::mir::value::{CollKind, NativeColl, Value};
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

// MatchType dispatch cost for collection equality. Charged first, before the
// length-mismatch short-circuit so the dispatch itself is always paid for.
const COLL_MATCH_TYPE_COST: u64 = 1;

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
            ctx.add_jit_cost(COLL_MATCH_TYPE_COST)?;
            let n = l_coll.len();
            if n != r_coll.len() {
                // Scala short-circuits on length mismatch without charging
                // per-item or base cost (bug 5).
                return Ok(false);
            }
            let (base, per_chunk, chunk_size) = coll_eq_cost(l_coll);
            ctx.add_per_item_jit_cost(base, per_chunk, chunk_size, n as u32)?;
            // COA leaf-element colls are bulk-compared (the per-item cost above
            // is the whole charge, mirroring JVM `equalCOA_*`). Composite-
            // element colls (Coll/Tuple/Option/SigmaProp -- the `coll_eq_cost`
            // default arm) recurse `eq_with_cost` per element, charging the
            // nested MatchType + per-item, as JVM's generic `equalColls` does
            // (DataValueComparer.scala:201-238).
            if is_coa_coll(l_coll) {
                Ok(lv == rv)
            } else {
                let l_items = l_coll.as_vec();
                let r_items = r_coll.as_vec();
                for (l, r) in l_items.iter().zip(r_items.iter()) {
                    if !eq_with_cost(l, r, ctx)? {
                        return Ok(false);
                    }
                }
                Ok(true)
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

        // SigmaProp, String, Unit, Lambda, Context, Global, and any cross-type
        // comparisons (which PartialEq returns false for anyway).
        _ => {
            ctx.add_jit_cost(EQ_PRIM_COST)?;
            Ok(lv == rv)
        }
    }
}

/// Per-item cost tuple for collection equality based on element type.
/// Mirrors Scala DataValueComparer's per-type array equality cost kinds.
fn coll_eq_cost(coll: &CollKind<Value<'_>>) -> (u32, u32, u32) {
    match coll {
        CollKind::NativeColl(NativeColl::CollByte(_)) => EQ_COLL_BYTE_PER_ITEM,
        CollKind::WrappedColl { elem_tpe, .. } => match elem_tpe {
            SType::SShort => EQ_COLL_SHORT_PER_ITEM,
            SType::SInt => EQ_COLL_INT_PER_ITEM,
            SType::SLong => EQ_COLL_LONG_PER_ITEM,
            SType::SBoolean => EQ_COLL_BOOLEAN_PER_ITEM,
            SType::SBigInt | SType::SUnsignedBigInt => EQ_COLL_BIGINT_PER_ITEM,
            SType::SGroupElement => EQ_COLL_GROUP_ELEMENT_PER_ITEM,
            SType::SAvlTree => EQ_COLL_AVL_TREE_PER_ITEM,
            SType::SBox => EQ_COLL_BOX_PER_ITEM,
            SType::SPreHeader => EQ_COLL_PREHEADER_PER_ITEM,
            SType::SHeader => EQ_COLL_HEADER_PER_ITEM,
            _ => EQ_COLL_DEFAULT_PER_ITEM,
        },
    }
}

/// Whether a collection's element type is a COA (CollOverArray) leaf — the
/// types JVM `equalColls_Dispatch` bulk-compares without recursion. This is
/// exactly the set with an explicit (non-default) arm in `coll_eq_cost`;
/// composite element types (the default arm: Coll/Tuple/Option/SigmaProp, …)
/// instead recurse `eq_with_cost` per element.
fn is_coa_coll(coll: &CollKind<Value<'_>>) -> bool {
    match coll {
        CollKind::NativeColl(NativeColl::CollByte(_)) => true,
        CollKind::WrappedColl { elem_tpe, .. } => matches!(
            elem_tpe,
            SType::SShort
                | SType::SInt
                | SType::SLong
                | SType::SBoolean
                | SType::SBigInt
                | SType::SUnsignedBigInt
                | SType::SGroupElement
                | SType::SAvlTree
                | SType::SBox
                | SType::SPreHeader
                | SType::SHeader
        ),
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
        assert_eq!(ctx.jit_cost_value() - before, COLL_MATCH_TYPE_COST);
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
}
