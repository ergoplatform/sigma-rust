//! Cost accumulator for JIT costing during ErgoTree evaluation.

use ergotree_ir::chain::context::Context;
use thiserror::Error;

use super::costs::{FixedCost, JitCost, PerItemCost};

/// Errors arising from cost accumulation.
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum CostError {
    /// Evaluation cost exceeded the allowed limit.
    #[error("Cost limit ({0}) exceeded")]
    LimitExceeded(u64),
}

/// Add a raw JitCost to the context's cost accumulator.
/// Returns CostError if the cost limit is exceeded.
#[inline]
pub fn add_cost(ctx: &Context, cost: JitCost) -> Result<(), CostError> {
    let new_cost = ctx.jit_cost_accum.get() + cost.0 as u64;
    if let Some(limit) = ctx.jit_cost_limit {
        if new_cost > limit {
            return Err(CostError::LimitExceeded(limit));
        }
    }
    ctx.jit_cost_accum.set(new_cost);
    Ok(())
}

/// Charge a fixed cost to the context.
#[inline]
pub fn add_fixed_cost(ctx: &Context, cost: FixedCost) -> Result<(), CostError> {
    add_cost(ctx, cost.0)
}

/// Charge a per-item cost to the context.
#[inline]
pub fn add_seq_cost(ctx: &Context, cost: PerItemCost, n_items: u32) -> Result<(), CostError> {
    add_cost(ctx, cost.total_cost(n_items))
}
