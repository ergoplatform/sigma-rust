//! Sigma byte stream writer
use crate::ergo_tree::ErgoTreeVersion;

use super::constant_store::ConstantStore;
use core3::io::Write;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;

/// Implementation for SigmaByteWrite
pub struct SigmaByteWriter<'a, W> {
    inner: &'a mut W,
    tree_version: ErgoTreeVersion,
    /// Constant store where constants (swapped for placeholders) are stored
    pub constant_store: Option<ConstantStore>,
}

impl<'a, W: Write> SigmaByteWriter<'a, W> {
    /// Make a new writer with underlying Write and optional constant store
    pub fn new(w: &'a mut W, constant_store: Option<ConstantStore>) -> SigmaByteWriter<'a, W> {
        SigmaByteWriter {
            inner: w,
            tree_version: ErgoTreeVersion::V0,
            constant_store,
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
}

impl<'a, W: Write> Write for SigmaByteWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> core3::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> core3::io::Result<()> {
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
}
