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
const EQ_PRIM_COST: u32 = 3;
const EQ_BIGINT_COST: u32 = 5;
const EQ_GROUP_ELEMENT_COST: u32 = 172;
const EQ_TUPLE_COST: u32 = 4;
const EQ_OPTION_COST: u32 = 4;
const EQ_AVL_TREE_COST: u32 = 6;
const EQ_BOX_COST: u32 = 6;
const EQ_PREHEADER_COST: u32 = 4;
const EQ_HEADER_COST: u32 = 6;

// MatchType dispatch cost for collection equality. Charged first, before the
// length-mismatch short-circuit so the dispatch itself is always paid for.
const COLL_MATCH_TYPE_COST: u32 = 1;

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
            Ok(lv == rv)
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
        assert_eq!(ctx.jit_cost_value() - before, EQ_PRIM_COST as u64);
    }

    #[test]
    fn coll_eq_charges_match_type_plus_per_item() {
        // Bug 4 regression: equal Coll[Int] of length 3 must pay MatchType(1)
        // + per-item SInt cost (base=15, per_chunk=2, chunk_size=64 → 1 chunk
        // of 3 items = 15 + 2 = 17). Total = 1 + 17 = 18.
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let items: Arc<[Value<'_>]> =
            Arc::from(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
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
        assert_eq!(ctx.jit_cost_value() - before, COLL_MATCH_TYPE_COST as u64);
    }
}
