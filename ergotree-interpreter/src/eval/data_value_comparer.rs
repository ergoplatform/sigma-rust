//! Per-type equality comparison with JIT costing.
//!
//! Mirrors Scala's `DataValueComparer` — each type charges a specific cost
//! before comparing values.

use ergotree_ir::chain::context::Context;
use ergotree_ir::mir::value::{CollKind, NativeColl, Value};
use ergotree_ir::types::stype::SType;

use super::cost_accum::{add_cost, add_fixed_cost, add_seq_cost, CostError};
use super::costs;

/// Compare two Values for equality, charging per-type costs to `ctx`.
/// Returns `Ok(true)` / `Ok(false)`, or `Err(CostError)` if the cost limit is exceeded.
pub fn eq_with_cost(lv: &Value<'_>, rv: &Value<'_>, ctx: &Context<'_>) -> Result<bool, CostError> {
    match (lv, rv) {
        // --- Primitives ---
        (Value::Boolean(_), Value::Boolean(_))
        | (Value::Byte(_), Value::Byte(_))
        | (Value::Short(_), Value::Short(_))
        | (Value::Int(_), Value::Int(_))
        | (Value::Long(_), Value::Long(_)) => {
            add_fixed_cost(ctx, costs::EQ_PRIM_COST)?;
            Ok(lv == rv)
        }

        // --- BigInt / UnsignedBigInt ---
        (Value::BigInt(_), Value::BigInt(_))
        | (Value::UnsignedBigInt(_), Value::UnsignedBigInt(_)) => {
            add_fixed_cost(ctx, costs::EQ_BIGINT_COST)?;
            Ok(lv == rv)
        }

        // --- GroupElement ---
        (Value::GroupElement(_), Value::GroupElement(_)) => {
            add_fixed_cost(ctx, costs::EQ_GROUP_ELEMENT_COST)?;
            Ok(lv == rv)
        }

        // --- Box ---
        (Value::CBox(_), Value::CBox(_)) => {
            add_fixed_cost(ctx, costs::EQ_BOX_COST)?;
            Ok(lv == rv)
        }

        // --- AvlTree ---
        (Value::AvlTree(_), Value::AvlTree(_)) => {
            add_fixed_cost(ctx, costs::EQ_AVL_TREE_COST)?;
            Ok(lv == rv)
        }

        // --- Tuple: fixed cost + recursive on elements ---
        (Value::Tup(l_items), Value::Tup(r_items)) => {
            add_fixed_cost(ctx, costs::EQ_TUPLE_COST)?;
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

        // --- Option: fixed cost + recursive on contents ---
        (Value::Opt(l_opt), Value::Opt(r_opt)) => {
            add_fixed_cost(ctx, costs::EQ_OPTION_COST)?;
            match (l_opt, r_opt) {
                (None, None) => Ok(true),
                (Some(l), Some(r)) => eq_with_cost(l, r, ctx),
                _ => Ok(false),
            }
        }

        // --- Collection: per-item cost based on element type ---
        (Value::Coll(l_coll), Value::Coll(r_coll)) => {
            // MatchType dispatch cost (matches Scala DataValueComparer case 2)
            add_cost(ctx, costs::JitCost(1))?;
            let n = l_coll.len();
            if n != r_coll.len() {
                // Scala returns false immediately after MatchType when lengths differ,
                // without charging any per-item or base cost.
                return Ok(false);
            }
            let per_item_cost = coll_eq_cost(l_coll);
            add_seq_cost(ctx, per_item_cost, n as u32)?;
            Ok(lv == rv)
        }

        // --- Header ---
        (Value::Header(_), Value::Header(_)) => {
            add_fixed_cost(ctx, costs::EQ_HEADER_COST)?;
            Ok(lv == rv)
        }

        // --- PreHeader ---
        (Value::PreHeader(_), Value::PreHeader(_)) => {
            add_fixed_cost(ctx, costs::EQ_PREHEADER_COST)?;
            Ok(lv == rv)
        }

        // --- SigmaProp, String, Unit, Lambda, etc. ---
        // Fall back to a primitive cost for types that are compared structurally
        _ => {
            add_fixed_cost(ctx, costs::EQ_PRIM_COST)?;
            Ok(lv == rv)
        }
    }
}

/// Returns the per-item cost for collection equality based on element type.
/// Mirrors Scala DataValueComparer's per-type array equality cost kinds.
fn coll_eq_cost(coll: &CollKind<Value<'_>>) -> costs::PerItemCost {
    match coll {
        CollKind::NativeColl(NativeColl::CollByte(_)) => costs::EQ_COLL_BYTE_PER_ITEM,
        CollKind::WrappedColl { elem_tpe, .. } => match elem_tpe {
            SType::SShort => costs::EQ_COLL_SHORT_PER_ITEM,
            SType::SInt => costs::EQ_COLL_INT_PER_ITEM,
            SType::SLong => costs::EQ_COLL_LONG_PER_ITEM,
            SType::SBoolean => costs::EQ_COLL_BOOLEAN_PER_ITEM,
            SType::SBigInt | SType::SUnsignedBigInt => costs::EQ_COLL_BIGINT_PER_ITEM,
            SType::SGroupElement => costs::EQ_COLL_GROUP_ELEMENT_PER_ITEM,
            SType::SAvlTree => costs::EQ_COLL_AVL_TREE_PER_ITEM,
            SType::SBox => costs::EQ_COLL_BOX_PER_ITEM,
            SType::SPreHeader => costs::EQ_COLL_PREHEADER_PER_ITEM,
            SType::SHeader => costs::EQ_COLL_HEADER_PER_ITEM,
            _ => costs::EQ_COLL_DEFAULT_PER_ITEM,
        },
    }
}
