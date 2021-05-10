//! Sigma byte stream writer
use super::constant_store::ConstantStore;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::io::Write;

/// Implementation for SigmaByteWrite
pub struct SigmaByteWriter<'a, W> {
    inner: &'a mut W,
    /// Constant store where constants (swapped for placeholders) are stored
    pub constant_store: Option<ConstantStore>,
}

impl<'a, W: Write> SigmaByteWriter<'a, W> {
    /// Make a new writer with underlying Write and optional constant store
    pub fn new(w: &'a mut W, constant_store: Option<ConstantStore>) -> SigmaByteWriter<'a, W> {
        SigmaByteWriter {
            inner: w,
            constant_store,
        }
    }
}

/// Sigma byte writer trait with a store for constant segregation
pub trait SigmaByteWrite: WriteSigmaVlqExt {
    /// Constant store (if any) attached to the writer to collect segregated constants
    fn constant_store_mut_ref(&mut self) -> Option<&mut ConstantStore>;
}

impl<'a, W: Write> Write for SigmaByteWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<'a, W: Write> SigmaByteWrite for SigmaByteWriter<'a, W> {
    fn constant_store_mut_ref(&mut self) -> Option<&mut ConstantStore> {
        self.constant_store.as_mut()
    }
}
