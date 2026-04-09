#[allow(dead_code)] // Used by JIT costing, wired incrementally

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

/// Compute per-item cost: base + ceil(n_items / chunk_size) * per_chunk
#[allow(dead_code)]
pub fn per_item_cost(base: u32, per_chunk: u32, chunk_size: u32, n_items: u32) -> u32 {
    let chunks = (n_items + chunk_size - 1) / chunk_size; // ceiling division
    base + chunks * per_chunk
}
