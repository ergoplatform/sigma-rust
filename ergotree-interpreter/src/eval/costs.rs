extern crate derive_more;
use derive_more::{From, Into};

/// JIT cost unit. Values are in 10x scale relative to block costs.
/// To convert to block cost: divide by 10.
#[derive(PartialEq, Eq, Debug, Clone, Copy, From, Into)]
pub struct JitCost(pub u32);

impl JitCost {
    /// Convert JIT cost to block cost (divides by 10, rounding down)
    pub fn to_block_cost(self) -> u64 {
        self.0 as u64 / 10
    }
}

/// Compute per-item cost: `base + chunks(n_items) * per_chunk`, where
/// `chunks(n)` mirrors Scala consensus `PerItemCost.chunks`:
/// `(n - 1) / chunk_size + 1` (signed, toward-zero division). Kept in lockstep
/// with `Context::add_per_item_jit_cost` so wiring this helper in later cannot
/// reintroduce the n=0 undercharge. Equals `ceil(n / chunk_size)` for n >= 1.
#[allow(dead_code)]
pub fn per_item_cost(base: u32, per_chunk: u32, chunk_size: u32, n_items: u32) -> u32 {
    let chunks = ((n_items as i64 - 1) / chunk_size as i64 + 1).max(0) as u32;
    base + chunks * per_chunk
}
