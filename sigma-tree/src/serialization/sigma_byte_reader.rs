use super::constant_store::ConstantStore;
use sigma_ser::{peekable_reader::Peekable, vlq_encode::ReadSigmaVlqExt};
use std::io::Read;

pub struct SigmaByteReader<R> {
    inner: R,
    constant_store: ConstantStore,
}

impl<R: Peekable> SigmaByteReader<R> {
    /// Create new reader from PeekableReader
    pub fn new(pr: R) -> SigmaByteReader<R> {
        SigmaByteReader {
            inner: pr,
            constant_store: ConstantStore::empty(),
        }
    }
}

pub trait SigmaByteRead: ReadSigmaVlqExt {
    fn constant_store(&mut self) -> &mut ConstantStore;
}

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

impl<R: ReadSigmaVlqExt> SigmaByteRead for SigmaByteReader<R> {
    fn constant_store(&mut self) -> &mut ConstantStore {
        &mut self.constant_store
    }
}
