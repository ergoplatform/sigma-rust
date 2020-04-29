use super::zig_zag_encode;
use std::convert::TryFrom;
use std::io;
use std::io::prelude::*;

#[cfg(test)]
use proptest::{num::u64, prelude::*};

/// Ways VLQ encoding/decoding might fail
#[derive(Debug)]
pub enum VlqEncodingError {
    /// IO fail (EOF, etc.)
    Io(io::Error),
    /// value bounds check error
    TryFrom(std::num::TryFromIntError),
    /// Fail to decode a value from bytes
    VlqDecodingFailed,
}

impl From<io::Error> for VlqEncodingError {
    fn from(error: io::Error) -> Self {
        VlqEncodingError::Io(error)
    }
}

impl From<std::num::TryFromIntError> for VlqEncodingError {
    fn from(error: std::num::TryFromIntError) -> Self {
        VlqEncodingError::TryFrom(error)
    }
}

/// Write encoded unsigned values using VLQ and signed values first with ZigZag, then using VLQ
/// for VLQ see [[https://en.wikipedia.org/wiki/Variable-length_quantity]]
/// for ZigZag see https://developers.google.com/protocol-buffers/docs/encoding#types
pub trait WriteSigmaVlqExt: io::Write {
    /// Write i8 without encoding
    fn put_i8(&mut self, v: i8) -> io::Result<()> {
        Self::put_u8(self, v as u8)
    }

    /// Write u8 without encoding
    fn put_u8(&mut self, v: u8) -> io::Result<()> {
        self.write_all(&[v])
    }

    /// Encode using ZigZag and then VLQ.
    fn put_i16(&mut self, v: i16) -> io::Result<()> {
        Self::put_u32(self, zig_zag_encode::encode_i32(v as i32))
    }

    /// Encode using VLQ.
    fn put_u16(&mut self, v: u16) -> io::Result<()> {
        Self::put_u64(self, v as u64)
    }

    /// Cast to u16 (with range check) and encode using VLQ
    fn put_usize_as_u16(&mut self, v: usize) -> io::Result<()> {
        Self::put_u16(self, u16::try_from(v).unwrap())
    }

    /// Encode using ZigZag and then VLQ.
    fn put_i32(&mut self, v: i32) -> io::Result<()> {
        Self::put_u64(self, zig_zag_encode::encode_i32(v as i32) as u64)
    }

    /// Encode using VLQ.
    fn put_u32(&mut self, v: u32) -> io::Result<()> {
        Self::put_u64(self, v as u64)
    }

    /// Encode using ZigZag and then VLQ.
    fn put_i64(&mut self, v: i64) -> io::Result<()> {
        Self::put_u64(self, zig_zag_encode::encode_i64(v))
    }

    /// Encode using VLQ.
    fn put_u64(&mut self, v: u64) -> io::Result<()> {
        let mut buffer: [u8; 10] = [0; 10];
        let mut position = 0;
        let mut value = v;
        // from https://github.com/ScorexFoundation/scorex-util/blob/3dc334f68ebefbfab6d33b57f2373e80245ab34d/src/main/scala/scorex/util/serialization/VLQWriter.scala#L97-L117
        // original source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedOutputStream.java#L1387
        // see https://rosettacode.org/wiki/Variable-length_quantity for implementations in other languages
        loop {
            if (value & !0x7F) == 0 {
                buffer[position] = value as u8;
                position += 1;
                break;
            } else {
                buffer[position] = (((value as u32) & 0x7F) | 0x80) as u8;
                position += 1;
                value >>= 7;
            }
        }
        Self::write_all(self, &buffer[..position])
    }
}

/// Mark all types implementing `Write` as implementing the extension.
impl<W: io::Write + ?Sized> WriteSigmaVlqExt for W {}

/// Read and decode values using VLQ (+ ZigZag for signed values) encoded and written with [`WriteSigmaVlqExt`]
/// for VLQ see [[https://en.wikipedia.org/wiki/Variable-length_quantity]]
/// for ZigZag see https://developers.google.com/protocol-buffers/docs/encoding#types
pub trait ReadSigmaVlqExt: io::Read {
    /// Peek a u8 without advancing the position
    fn peek_u8(&mut self) -> Result<u8, io::Error> {
        // TODO: fix and enable test, implementation below does not work (consumes the reader)
        let mut slice = [0u8; 1];
        self.take(1).read(&mut slice)?;
        Ok(slice[0])
    }

    /// Read i8 without decoding
    fn get_i8(&mut self) -> Result<i8, io::Error> {
        Self::get_u8(self).map(|v| v as i8)
    }

    /// Read u8 without decoding
    fn get_u8(&mut self) -> std::result::Result<u8, io::Error> {
        let mut slice = [0u8; 1];
        self.read_exact(&mut slice)?;
        Ok(slice[0])
    }

    /// Read and decode using VLQ and ZigZag value written with [`WriteSigmaVlqExt::put_i16`]
    fn get_i16(&mut self) -> Result<i16, VlqEncodingError> {
        Self::get_u32(self).and_then(|v| {
            let vd = zig_zag_encode::decode_u32(v);
            i16::try_from(vd).map_err(VlqEncodingError::TryFrom)
        })
    }

    /// Read and decode using VLQ value written with [`WriteSigmaVlqExt::put_u16`]
    fn get_u16(&mut self) -> Result<u16, VlqEncodingError> {
        Self::get_u64(self).and_then(|v| u16::try_from(v).map_err(VlqEncodingError::TryFrom))
    }

    /// Read and decode using VLQ and ZigZag value written with [`WriteSigmaVlqExt::put_i32`]
    fn get_i32(&mut self) -> Result<i32, VlqEncodingError> {
        Self::get_u64(self)
            .and_then(|v| u32::try_from(v).map_err(VlqEncodingError::TryFrom))
            .map(zig_zag_encode::decode_u32)
    }

    /// Read and decode using VLQ value written with [`WriteSigmaVlqExt::put_u32`]
    fn get_u32(&mut self) -> Result<u32, VlqEncodingError> {
        Self::get_u64(self).and_then(|v| u32::try_from(v).map_err(VlqEncodingError::TryFrom))
    }

    /// Read and decode using VLQ and ZigZag value written with [`WriteSigmaVlqExt::put_i64`]
    fn get_i64(&mut self) -> Result<i64, VlqEncodingError> {
        Self::get_u64(self).map(zig_zag_encode::decode_u64)
    }

    /// Read and decode using VLQ value written with [`WriteSigmaVlqExt::put_u64`]
    fn get_u64(&mut self) -> Result<u64, VlqEncodingError> {
        // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedInputStream.java#L2653
        // for faster version see: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedInputStream.java#L1085
        // see https://rosettacode.org/wiki/Variable-length_quantity for implementations in other languages
        let mut result: i64 = 0;
        let mut shift = 0;
        while shift < 64 {
            let b = self.get_u8()?;
            result |= ((b & 0x7F) as i64) << shift;
            if (b & 0x80) == 0 {
                return Ok(result as u64);
            }
            shift += 7;
        }
        Err(VlqEncodingError::VlqDecodingFailed)
    }
}

/// Mark all types implementing `Read` as implementing the extension.
impl<R: io::Read + ?Sized> ReadSigmaVlqExt for R {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_u8() {
        let mut w = Cursor::new(vec![]);
        w.put_u8(0).unwrap();
        w.put_u8(1).unwrap();
        w.put_u8(255).unwrap();

        assert_eq!(w.into_inner(), vec![0, 1, 255])
    }

    #[test]
    fn test_read_u8() {
        let mut r = Cursor::new(vec![0, 1, 255]);
        assert_eq!(r.get_u8().unwrap(), 0);
        assert_eq!(r.get_u8().unwrap(), 1);
        assert_eq!(r.get_u8().unwrap(), 255);
    }

    #[ignore]
    #[test]
    fn test_peek_u8() {
        let mut r = Cursor::new(vec![0, 1]);
        assert_eq!(r.peek_u8().unwrap(), 0);
        assert_eq!(r.get_u8().unwrap(), 0);
        assert_eq!(r.peek_u8().unwrap(), 1);
        assert_eq!(r.get_u8().unwrap(), 1);
    }

    // from https://github.com/ScorexFoundation/scorex-util/blob/3dc334f68ebefbfab6d33b57f2373e80245ab34d/src/test/scala/scorex/util/serialization/VLQReaderWriterSpecification.scala#L32-L32
    // original source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/test/java/com/google/protobuf/CodedInputStreamTest.java#L239
    #[allow(clippy::identity_op)]
    fn expected_values() -> Vec<(Vec<u8>, u64)> {
        vec![
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x7f], 127),
            // 14882
            (vec![0xa2, 0x74], (0x22 << 0) | (0x74 << 7)),
            // 2961488830
            (
                vec![0xbe, 0xf7, 0x92, 0x84, 0x0b],
                (0x3e << 0) | (0x77 << 7) | (0x12 << 14) | (0x04 << 21) | (0x0b << 28),
            ),
            // 64-bit
            // 7256456126
            (
                vec![0xbe, 0xf7, 0x92, 0x84, 0x1b],
                (0x3e << 0) | (0x77 << 7) | (0x12 << 14) | (0x04 << 21) | (0x1b << 28),
            ),
            // 41256202580718336
            (
                vec![0x80, 0xe6, 0xeb, 0x9c, 0xc3, 0xc9, 0xa4, 0x49],
                (0x00 << 0)
                    | (0x66 << 7)
                    | (0x6b << 14)
                    | (0x1c << 21)
                    | (0x43 << 28)
                    | (0x49 << 35)
                    | (0x24 << 42)
                    | (0x49 << 49),
            ),
            // 11964378330978735131 (-6482365742730816485)
            (
                vec![0x9b, 0xa8, 0xf9, 0xc2, 0xbb, 0xd6, 0x80, 0x85, 0xa6, 0x01],
                (0x1b << 0)
                    | (0x28 << 7)
                    | (0x79 << 14)
                    | (0x42 << 21)
                    | (0x3b << 28)
                    | (0x56 << 35)
                    | (0x00 << 42)
                    | (0x05 << 49)
                    | (0x26 << 56)
                    | (0x01 << 63),
            ),
        ]
    }

    #[test]
    fn test_write_u64_expected_values() {
        for pair in expected_values() {
            let (bytes, value) = pair;
            let mut w = Cursor::new(vec![]);
            w.put_u64(value).unwrap();
            assert_eq!(w.into_inner(), bytes)
        }
    }

    #[test]
    fn test_read_u64_expected_values() {
        for pair in expected_values() {
            let (bytes, value) = pair;
            let mut r = Cursor::new(bytes);
            let decoded_value = r.get_u64().unwrap();
            assert_eq!(decoded_value, value)
        }
    }

    #[cfg(test)]
    proptest! {

        #[test]
        fn prop_u64_roundtrip(i in u64::ANY) {
            let mut w = Cursor::new(vec![]);
            w.put_u64(i).unwrap();
            let mut r = Cursor::new(w.into_inner());
            prop_assert_eq![i, r.get_u64().unwrap()];
        }

        #[test]
        fn prop_u64_array_roundtrip(arr in any::<[u64; 32]>()) {
            let mut w = Cursor::new(vec![]);
            for a in arr.iter() {
                w.put_u64(*a).unwrap();
            }
            let mut dec = Vec::new();
            let mut r = Cursor::new(w.into_inner());
            for _ in 0..arr.len() {
                dec.push(r.get_u64().unwrap());
            }
            prop_assert_eq![dec, arr];
        }
    }
}
