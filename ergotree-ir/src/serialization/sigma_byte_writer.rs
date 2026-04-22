//! Sigma byte stream writer
use crate::ergo_tree::ErgoTreeVersion;

use super::constant_store::ConstantStore;
use core::cell::Cell;
use core2::io::Write;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;

// Per-write-operation cost constants (from Scala SigmaByteWriter companion object).
// These are JitCost values used during instrumented serialization.

/// Cost charged once when starting a new writer.
pub const START_WRITER_COST: u32 = 10;
/// Cost per single-byte write (put_u8, put_i8, putBoolean).
pub const PUT_BYTE_COST: u32 = 1;
/// Cost per signed numeric VLQ write (put_i16, put_i32, put_i64).
pub const PUT_SIGNED_NUMERIC_COST: u32 = 3;
/// Cost per unsigned numeric VLQ write (put_u16, put_u32, put_u64).
pub const PUT_UNSIGNED_NUMERIC_COST: u32 = 3;
/// Base cost for a chunk/byte-array write (putBytes, putBits, putShortString).
pub const PUT_CHUNK_BASE_COST: u32 = 3;
/// Per-byte cost within a chunk write. Total = PUT_CHUNK_BASE_COST + n_bytes * PUT_CHUNK_PER_BYTE.
pub const PUT_CHUNK_PER_BYTE: u32 = 1;

/// Implementation for SigmaByteWrite
pub struct SigmaByteWriter<'a, W> {
    inner: &'a mut W,
    tree_version: ErgoTreeVersion,
    /// Constant store where constants (swapped for placeholders) are stored
    pub constant_store: Option<ConstantStore>,
    /// Optional JIT cost accumulator for instrumented serialization
    cost_accum: Option<&'a Cell<u64>>,
}

impl<'a, W: Write> SigmaByteWriter<'a, W> {
    /// Make a new writer with underlying Write and optional constant store
    pub fn new(w: &'a mut W, constant_store: Option<ConstantStore>) -> SigmaByteWriter<'a, W> {
        SigmaByteWriter {
            inner: w,
            tree_version: ErgoTreeVersion::V0,
            constant_store,
            cost_accum: None,
        }
    }

    /// Make a new writer with JIT cost accumulation for instrumented serialization.
    /// The cost accumulator is incremented by per-operation costs during serialization.
    pub fn new_with_cost(
        w: &'a mut W,
        constant_store: Option<ConstantStore>,
        cost_accum: &'a Cell<u64>,
    ) -> SigmaByteWriter<'a, W> {
        SigmaByteWriter {
            inner: w,
            tree_version: ErgoTreeVersion::V0,
            constant_store,
            cost_accum: Some(cost_accum),
        }
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

    /// Charge a write cost to the JIT cost accumulator (if present).
    /// Default implementation is a no-op for writers without cost tracking.
    fn charge_write_cost(&mut self, _cost: u32) {}
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
    fn charge_write_cost(&mut self, cost: u32) {
        if let Some(accum) = &self.cost_accum {
            accum.set(accum.get() + cost as u64);
        }
    }
}
