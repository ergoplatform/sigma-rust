use super::constant_store::ConstantStore;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::io::Write;

pub struct SigmaByteWriter<'a, W> {
    inner: &'a mut W,
    constant_store: Option<ConstantStore>,
}

impl<'a, W: Write> SigmaByteWriter<'a, W> {
    pub fn new(w: &mut W) -> SigmaByteWriter<W> {
        SigmaByteWriter {
            inner: w,
            constant_store: None,
        }
    }
}

pub trait SigmaByteWrite: WriteSigmaVlqExt {
    fn constant_store(&mut self) -> Option<&mut ConstantStore>;
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
    fn constant_store(&mut self) -> Option<&mut ConstantStore> {
        match self.constant_store.as_mut() {
            Some(store) => Some(store),
            None => None,
        }
    }
}
