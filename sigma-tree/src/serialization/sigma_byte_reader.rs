use super::constant_store::ConstantStore;
use sigma_ser::{peekable_reader::Peekable, vlq_encode::ReadSigmaVlqExt};
use std::io::Read;

/// Expose constant store
pub trait ConstantStoreHolder {
    /// Constant store
    fn constant_store(&mut self) -> Option<&mut ConstantStore>;
}

/// Sigma reader
pub struct SigmaByteReader<R> {
    inner: R,
    constant_store: Option<ConstantStore>,
}

impl<R: Peekable> SigmaByteReader<R> {
    /// Create new reader from PeekableReader
    pub fn new(pr: R) -> SigmaByteReader<R> {
        SigmaByteReader {
            inner: pr,
            constant_store: Some(ConstantStore::empty()),
        }
    }

    // pub fn new2(pr: R) -> Box<dyn SigmaByteRead> {
    //     Box::new(SigmaByteReader {
    //         inner: pr,
    //         constant_store: ConstantStore::empty(),
    //     })
    // }
}

impl<R: ReadSigmaVlqExt> ConstantStoreHolder for SigmaByteReader<R> {
    fn constant_store(&mut self) -> Option<&mut ConstantStore> {
        match self.constant_store.as_mut() {
            Some(store) => Some(store),
            None => None,
        }
    }
}

/// Compaund trait for sigma byte reader (VLQ, Peekable, ConstantStore)
pub trait SigmaByteRead: ReadSigmaVlqExt + ConstantStoreHolder {}

impl<R: Peekable> Read for SigmaByteReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Peekable> Peekable for SigmaByteReader<R> {
    fn peek_u8(&mut self) -> Result<u8, &std::io::Error> {
        self.inner.peek_u8()
    }
}

impl<R: Peekable> SigmaByteRead for SigmaByteReader<R> {}
