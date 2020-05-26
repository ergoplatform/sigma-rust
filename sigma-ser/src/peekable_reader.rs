use std::io::{Error, Read};
use std::mem::swap;

/// from https://github.com/C4K3/peekable-reader-rs/blob/master/src/lib.rs

/// A wrapper around any struct implementing the `Read` trait, additionally
/// allowing for `peek` operations to be performed. Therefore, the
/// `PeekableReader` struct also implements the `Read` trait.
///
/// The primary invariant of the `PeekableReader` is that after calling the
/// `peek` method, the next `read_byte` call will return the same result as
/// the `peek` does. When the result is a byte (read off the wrapped reader),
/// any read-type method of the `Reader` trait will include the byte as the
/// first one. On the other hand, if the result is an `io::Error`, the error
/// will be returned regardless of which read-type method of the `Reader` is
/// invoked. Consecutive `peek`s before any read-type operation is used
/// always return the same `io::Result`.
///
/// When using peek_u8 you will get a Result<u8, &Error>, while when you call
/// any of the `Read` functions you will get an io::Result<u8>, thereby
/// consuming the io::Error (if any.)
pub struct PeekableReader<R> {
    inner: R,
    peeked_result: Option<Result<u8, Error>>,
}

/// A wrapper around any struct implementing the `Read` trait, additionally
/// allowing for `peek` operations to be performed. Therefore, the
/// `PeekableReader` struct also implements the `Read` trait.
///
/// The primary invariant of the `PeekableReader` is that after calling the
/// `peek` method, the next `read_byte` call will return the same result as
/// the `peek` does. When the result is a byte (read off the wrapped reader),
/// any read-type method of the `Reader` trait will include the byte as the
/// first one. On the other hand, if the result is an `io::Error`, the error
/// will be returned regardless of which read-type method of the `Reader` is
/// invoked. Consecutive `peek`s before any read-type operation is used
/// always return the same `io::Result`.
///
pub trait Peekable: Read {
    /// When using peek_u8 you will get a Result<u8, &Error>, while when you call
    /// any of the `Read` functions you will get an io::Result<u8>, thereby
    /// consuming the io::Error (if any.)
    fn peek_u8(&mut self) -> Result<u8, &Error>;
}

impl<R: Read> Peekable for PeekableReader<R> {
    /// Returns the `io::Result` which the Reader will return on the next
    /// `get_byte` call.
    ///
    /// If the `io::Result` is indeed an `io::Error`, the error will be returned
    /// for any subsequent read operation invoked upon the `Read`er.
    fn peek_u8(&mut self) -> Result<u8, &Error> {
        // Return either the currently cached peeked byte or obtain a new one
        // from the underlying reader.
        match self.peeked_result {
            Some(Ok(x)) => Ok(x),
            Some(Err(ref e)) => Err(e),
            None => {
                // First get the result of the read from the underlying reader
                let mut tmp: [u8; 1] = [0];
                self.peeked_result = match self.inner.read_exact(&mut tmp) {
                    Ok(()) => Some(Ok(tmp[0])),
                    Err(e) => Some(Err(e)),
                };

                // Now just return that
                let tmp: Result<u8, &Error> = match self.peeked_result {
                    Some(Ok(x)) => Ok(x),
                    Some(Err(ref e)) => Err(e),
                    None => unreachable!(),
                };
                tmp
            }
        }
    }
}

impl<R: Read> PeekableReader<R> {
    /// Initializes a new `PeekableReader` which wraps the given underlying
    /// reader.
    pub fn new(reader: R) -> PeekableReader<R> {
        PeekableReader {
            inner: reader,
            peeked_result: None,
        }
    }
}

impl<R: Read> Read for PeekableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.len() == 0 {
            return Ok(0);
        }

        // First, put the byte that was read off the underlying reader in a
        // (possible) previous peek operation (if such a byte is indeed cached)
        let mut tmp = None;
        swap(&mut tmp, &mut self.peeked_result);
        let offset = match tmp {
            Some(Err(e)) => {
                return Err(e);
            }
            Some(Ok(b)) => {
                buf[0] = b;
                1
            }
            None => 0,
        };

        if offset == 1 && buf.len() == 1 {
            // We've filled the buffer by using the previously peeked byte
            Ok(1)
        } else {
            // We are still missing more bytes, so we read them from the
            // underlying reader and place them directly in the correct place
            // in the buffer.
            Ok((self.inner.read(&mut buf[offset..]))? + offset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vlq_encode::ReadSigmaVlqExt;
    use std::io::Cursor;

    #[test]
    fn test_peek_u8() {
        let mut r = PeekableReader::new(Cursor::new(vec![0, 1]));
        assert_eq!(r.peek_u8().unwrap(), 0);
        assert_eq!(r.get_u8().unwrap(), 0);
        assert_eq!(r.peek_u8().unwrap(), 1);
        assert_eq!(r.get_u8().unwrap(), 1);
    }
}
