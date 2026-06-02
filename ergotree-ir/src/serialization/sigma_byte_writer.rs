//! Sigma byte stream writer
use crate::ergo_tree::ErgoTreeVersion;

use super::constant_store::ConstantStore;
use core2::io::Write;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;

// Scala `Global.serialize` cost constants (sigma-state `SigmaByteWriter.scala:235-262`).
// `serialize_eval` charges these per writer `put` while serializing the value; sigma-rust
// accumulates them in `SigmaByteWriter::serialize_cost_accum` when cost tracking is enabled
// (see `SERIALIZE_EVAL_FN`). All are `JitCost` units (1:1 with the accumulator).
/// `PutByteCost = FixedCost(JitCost(1))` — `put`/`putBoolean`/option tag byte.
const PUT_BYTE_COST: u64 = 1;
/// `Put{Signed,Unsigned}NumericCost = FixedCost(JitCost(3))` — `putShort/Int/Long`, `putU*`.
const PUT_NUMERIC_COST: u64 = 3;
/// `PutChunkCost = PerItemCost(JitCost(3), JitCost(1), 1)` ⇒ `cost(n) = 3 + n` —
/// `putBytes`/`putBits`/`putChunk` over `n` items.
const PUT_CHUNK_BASE_COST: u64 = 3;

/// Implementation for SigmaByteWrite
pub struct SigmaByteWriter<'a, W> {
    inner: &'a mut W,
    tree_version: ErgoTreeVersion,
    /// Constant store where constants (swapped for placeholders) are stored
    pub constant_store: Option<ConstantStore>,
    /// When `Some`, accumulates per-`put` serialize cost (enabled by `Global.serialize`).
    serialize_cost_accum: Option<u64>,
}

impl<'a, W: Write> SigmaByteWriter<'a, W> {
    /// Make a new writer with underlying Write and optional constant store
    pub fn new(w: &'a mut W, constant_store: Option<ConstantStore>) -> SigmaByteWriter<'a, W> {
        SigmaByteWriter {
            inner: w,
            tree_version: ErgoTreeVersion::V0,
            constant_store,
            serialize_cost_accum: None,
        }
    }

    /// Enable per-`put` serialize cost tracking, starting the accumulator at 0.
    /// Used by `Global.serialize` to charge the writer's per-op cost (see `SERIALIZE_EVAL_FN`).
    pub fn enable_serialize_cost_tracking(&mut self) {
        self.serialize_cost_accum = Some(0);
    }

    /// Total per-`put` serialize cost accumulated since tracking was enabled (0 if disabled).
    pub fn serialize_cost(&self) -> u64 {
        self.serialize_cost_accum.unwrap_or(0)
    }
}

/// Sigma byte writer trait with a store for constant segregation
pub trait SigmaByteWrite: WriteSigmaVlqExt {
    /// Constant store (if any) attached to the writer to collect segregated constants
    fn constant_store_mut_ref(&mut self) -> Option<&mut ConstantStore>;
    /// ErgoTree Version
    fn tree_version(&self) -> ErgoTreeVersion;
    /// Execute `f` with ErgoTree version set to `version`
    fn with_tree_version<T>(
        &mut self,
        version: ErgoTreeVersion,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T;

    /// Record the cost of a fixed-cost byte `put` (Scala `PutByteCost`). No-op unless
    /// serialize cost tracking is enabled. Called from `DataSerializer::sigma_serialize`.
    fn add_put_byte_cost(&mut self) {}
    /// Record the cost of a numeric `put` (Scala `Put{Signed,Unsigned}NumericCost`). No-op
    /// unless serialize cost tracking is enabled.
    fn add_put_numeric_cost(&mut self) {}
    /// Record the cost of a chunk `put` of `n` items (Scala `PutChunkCost`, `cost(n) = 3 + n`).
    /// No-op unless serialize cost tracking is enabled.
    fn add_put_chunk_cost(&mut self, _n: usize) {}
}

impl<'a, W: Write> Write for SigmaByteWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> core2::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> core2::io::Result<()> {
        self.inner.flush()
    }
}

impl<'a, W: Write> SigmaByteWrite for SigmaByteWriter<'a, W> {
    fn constant_store_mut_ref(&mut self) -> Option<&mut ConstantStore> {
        self.constant_store.as_mut()
    }
    fn tree_version(&self) -> ErgoTreeVersion {
        self.tree_version
    }
    fn with_tree_version<T>(
        &mut self,
        version: ErgoTreeVersion,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let tmp = self.tree_version;
        self.tree_version = version;
        let res = f(self);
        self.tree_version = tmp;
        res
    }

    fn add_put_byte_cost(&mut self) {
        if let Some(c) = self.serialize_cost_accum.as_mut() {
            *c += PUT_BYTE_COST;
        }
    }
    fn add_put_numeric_cost(&mut self) {
        if let Some(c) = self.serialize_cost_accum.as_mut() {
            *c += PUT_NUMERIC_COST;
        }
    }
    fn add_put_chunk_cost(&mut self, n: usize) {
        if let Some(c) = self.serialize_cost_accum.as_mut() {
            *c += PUT_CHUNK_BASE_COST + n as u64;
        }
    }
}
