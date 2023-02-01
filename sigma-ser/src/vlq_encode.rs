use super::zig_zag_encode;
use std::convert::TryFrom;
use std::io;

use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
#[cfg(test)]
use proptest::{num::u64, prelude::*};
use thiserror::Error;

/// Ways VLQ encoding/decoding might fail
#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum VlqEncodingError {
    /// IO fail (EOF, etc.)
    #[error("IO error: {0}")]
    Io(String),
    /// value bounds check error
    #[error("Bounds check error: {1} for input: {0}")]
    TryFrom(String, std::num::TryFromIntError),
    /// Fail to decode a value from bytes
    #[error("VLQ decoding failed")]
    VlqDecodingFailed,
}

impl From<io::Error> for VlqEncodingError {
    fn from(error: io::Error) -> Self {
        VlqEncodingError::Io(error.to_string())
    }
}

/// Write encoded unsigned values using VLQ and signed values first with ZigZag, then using VLQ
/// for VLQ see <https://en.wikipedia.org/wiki/Variable-length_quantity> (GLE)
/// for ZigZag see <https://developers.google.com/protocol-buffers/docs/encoding#types>
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
        Self::put_u32(self, zig_zag_encode::encode_i32(v as i32) as u32)
    }

    /// Encode using VLQ.
    fn put_u16(&mut self, v: u16) -> io::Result<()> {
        Self::put_u64(self, v as u64)
    }

    /// Cast to u16 (panics if out of range) and encode using VLQ
    fn put_usize_as_u16_unwrapped(&mut self, v: usize) -> io::Result<()> {
        #[allow(clippy::unwrap_used)]
        Self::put_u16(self, u16::try_from(v).unwrap())
    }

    /// Cast to u32 (panics if out of range) and encode using VLQ
    fn put_usize_as_u32_unwrapped(&mut self, v: usize) -> io::Result<()> {
        #[allow(clippy::unwrap_used)]
        Self::put_u32(self, u32::try_from(v).unwrap())
    }

    /// Encode using ZigZag and then VLQ.
    fn put_i32(&mut self, v: i32) -> io::Result<()> {
        Self::put_u64(self, zig_zag_encode::encode_i32(v))
    }

    /// Encode using VLQ.
    fn put_u32(&mut self, v: u32) -> io::Result<()> {
        Self::put_u64(self, v as u64)
    }

    /// Write bytes of v directly (big-endian format)
    fn put_u32_be_bytes(&mut self, v: u32) -> io::Result<()> {
        self.write_all(&v.to_be_bytes())?;
        Ok(())
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
        loop {
            if (value & !0x7F) == 0 {
                buffer[position] = value as u8;
                position += 1;
                break;
            } else {
                buffer[position] = (((value as i32) & 0x7F) | 0x80) as u8;
                position += 1;
                value >>= 7;
            };
        }
        self.write_all(&buffer[..position])
    }

    /// Encode bool array as bit vector, filling trailing bits with `false`
    fn put_bits(&mut self, bools: &[bool]) -> io::Result<()> {
        let mut bits = BitVec::<u8, Lsb0>::new();
        for b in bools {
            bits.push(*b);
        }
        for c in bits.as_bitslice().domain() {
            self.put_u8(c)?;
        }
        Ok(())
    }

    /// Put the two bytes of the big-endian representation of the i16 value into the writer.
    fn put_i16_be_bytes(&mut self, v: i16) -> io::Result<()> {
        self.write_all(v.to_be_bytes().as_ref())
    }

    /// Put a short string (< 256 bytes) into the writer. Writes length (as u8) and string bytes to the writer
    fn put_short_string(&mut self, s: &str) -> io::Result<()> {
        if s.len() > 255 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Serializing strings with more than 255 bytes is not allowed",
            ));
        }
        self.put_u8(s.len() as u8)?;
        self.write_all(s.as_bytes())?;
        Ok(())
    }

    /// Encode an optional value
    fn put_option<T>(
        &mut self,
        opt: Option<T>,
        put_value: &dyn Fn(&mut Self, T) -> io::Result<()>,
    ) -> io::Result<()> {
        match opt {
            Some(s) => {
                self.put_u8(1)?;
                put_value(self, s)?;
            }
            None => self.put_u8(0)?,
        }
        Ok(())
    }
}

/// Mark all types implementing `Write` as implementing the extension.
impl<W: io::Write + ?Sized> WriteSigmaVlqExt for W {}

/// Read and decode values using VLQ (+ ZigZag for signed values) encoded and written with [`WriteSigmaVlqExt`]
/// for VLQ see <https://en.wikipedia.org/wiki/Variable-length_quantity> (GLE)
/// for ZigZag see <https://developers.google.com/protocol-buffers/docs/encoding#types>
pub trait ReadSigmaVlqExt: io::Read {
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
        Self::get_u64(self).and_then(|v| {
            let vd = zig_zag_encode::decode_u32(v);
            i16::try_from(vd).map_err(|err| VlqEncodingError::TryFrom(vd.to_string(), err))
        })
    }

    /// Read and decode using VLQ value written with [`WriteSigmaVlqExt::put_u16`]
    fn get_u16(&mut self) -> Result<u16, VlqEncodingError> {
        Self::get_u64(self).and_then(|v| {
            u16::try_from(v).map_err(|err| VlqEncodingError::TryFrom(v.to_string(), err))
        })
    }

    /// Read and decode using VLQ and ZigZag value written with [`WriteSigmaVlqExt::put_i32`]
    fn get_i32(&mut self) -> Result<i32, VlqEncodingError> {
        Self::get_u64(self).map(zig_zag_encode::decode_u32)
    }

    /// Read and decode using VLQ value written with [`WriteSigmaVlqExt::put_u32`]
    fn get_u32(&mut self) -> Result<u32, VlqEncodingError> {
        Self::get_u64(self).and_then(|v| {
            u32::try_from(v).map_err(|err| VlqEncodingError::TryFrom(v.to_string(), err))
        })
    }

    /// Read and decode using VLQ and ZigZag value written with [`WriteSigmaVlqExt::put_i64`]
    fn get_i64(&mut self) -> Result<i64, VlqEncodingError> {
        Self::get_u64(self).map(zig_zag_encode::decode_u64)
    }

    /// Read and decode using VLQ value written with [`WriteSigmaVlqExt::put_u64`]
    fn get_u64(&mut self) -> Result<u64, VlqEncodingError> {
        // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedInputStream.java#L2653
        // for faster version see: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedInputStream.java#L1085
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

    /// Read a vector of bits with the given size
    fn get_bits(&mut self, size: usize) -> Result<Vec<bool>, VlqEncodingError> {
        let byte_num = (size + 7) / 8;
        let mut buf = vec![0u8; byte_num];
        self.read_exact(&mut buf)?;
        // May fail if number of bits in buf is larger that maximum value of usize
        let mut bits = BitVec::<u8, Lsb0>::from_vec(buf);
        bits.truncate(size);
        Ok(bits.iter().map(|x| *x).collect::<Vec<bool>>())
    }

    /// Reads a string from the reader. Reads a byte (size), and the string
    fn get_short_string(&mut self) -> Result<String, VlqEncodingError> {
        let size_bytes = self.get_u8()?;
        let mut bytes = vec![0u8; size_bytes as usize];
        self.read_exact(&mut bytes)?;
        let string = String::from_utf8(bytes).map_err(|_| VlqEncodingError::VlqDecodingFailed)?;
        Ok(string)
    }

    /// Read and decode an optional value using supplied function
    fn get_option<T>(
        &mut self,
        get_value: &dyn Fn(&mut Self) -> Result<T, VlqEncodingError>,
    ) -> Option<T> {
        let is_opt = self.get_u8().ok()?;
        match is_opt {
            1 => Some(get_value(self).ok()?),
            // Should only ever be 0 or 1
            _ => None,
        }
    }
}

/// Mark all types implementing `Read` as implementing the extension.
impl<R: io::Read + ?Sized> ReadSigmaVlqExt for R {}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    // See corresponding test suite in
    // https://github.com/ScorexFoundation/scorex-util/blob/9adb6c68b8a1c00ec17730e6da11c2976a892ad8/src/test/scala/scorex/util/serialization/VLQReaderWriterSpecification.scala#L11
    use super::*;
    use proptest::collection;
    use std::io::Cursor;
    use std::io::Read;
    use std::io::Write;

    extern crate derive_more;
    use derive_more::From;

    #[derive(Debug, From, Clone, PartialEq)]
    enum Val {
        I8(i8),
        U8(u8),
        I16(i16),
        U16(u16),
        I32(i32),
        U32(u32),
        I64(i64),
        U64(u64),
        Bytes(Vec<u8>),
        Bits(Vec<bool>),
    }

    impl Arbitrary for Val {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any::<i8>().prop_map_into(),
                any::<u8>().prop_map_into(),
                any::<i16>().prop_map_into(),
                any::<u16>().prop_map_into(),
                any::<i32>().prop_map_into(),
                any::<u32>().prop_map_into(),
                any::<i64>().prop_map_into(),
                any::<u64>().prop_map_into(),
                any::<Vec<u8>>().prop_map_into(),
                any::<Vec<bool>>().prop_map_into(),
            ]
            .boxed()
        }
    }

    fn bytes_u64(v: u64) -> Vec<u8> {
        let mut w = Cursor::new(vec![]);
        w.put_u64(v).unwrap();
        w.into_inner()
    }

    fn bytes_i64(v: i64) -> Vec<u8> {
        let mut w = Cursor::new(vec![]);
        w.put_i64(v).unwrap();
        w.into_inner()
    }

    fn bytes_u32(v: u32) -> Vec<u8> {
        let mut w = Cursor::new(vec![]);
        w.put_u32(v).unwrap();
        w.into_inner()
    }

    fn bytes_i32(v: i32) -> Vec<u8> {
        let mut w = Cursor::new(vec![]);
        w.put_i32(v).unwrap();
        w.into_inner()
    }
    fn bytes_u16(v: u16) -> Vec<u8> {
        let mut w = Cursor::new(vec![]);
        w.put_u16(v).unwrap();
        w.into_inner()
    }

    fn bytes_i16(v: i16) -> Vec<u8> {
        let mut w = Cursor::new(vec![]);
        w.put_i16(v).unwrap();
        w.into_inner()
    }

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

    #[test]
    fn test_i32_ten_bytes_case() {
        let input = 1234567890i32;
        let mut w = Cursor::new(vec![]);
        w.put_i32(input).unwrap();
        let bytes = w.into_inner();
        assert_eq!(bytes.len(), 10);
        // 164, 139, 176, 153, 9,
        let mut r = Cursor::new(bytes);
        let decoded_value = r.get_i32().unwrap();
        assert_eq!(decoded_value, input);
    }

    #[test]
    fn malformed_input() {
        // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/test/java/com/google/protobuf/CodedInputStreamTest.java#L281
        assert!(Cursor::new([0x80]).get_u64().is_err());
        assert!(
            Cursor::new([0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00])
                .get_u64()
                .is_err()
        );
    }

    #[test]
    fn i16_corner_cases() {
        fn roundtrip(v: i16, expected_bytes: &[u8]) {
            let mut w = Cursor::new(vec![]);
            w.put_i16(v).unwrap();
            let bytes = w.into_inner();
            assert_eq!(bytes, expected_bytes);
            let mut r = Cursor::new(expected_bytes);
            let decoded_value = r.get_i16().unwrap();
            assert_eq!(decoded_value, v);
        }

        roundtrip(i16::MIN, &[0xFF, 0xFF, 0x03]);
        roundtrip(-8194, &[0x83, 0x80, 0x01]);
        roundtrip(-8193, &[0x81, 0x80, 0x01]);
        roundtrip(-8192, &[0xFF, 0x7F]);
        roundtrip(-8191, &[0xFD, 0x7F]);
        roundtrip(-66, &[0x83, 0x01]);
        assert_eq!(Cursor::new([0x83, 0x00]).get_i16().unwrap(), -2);
        roundtrip(-65, &[0x81, 0x01]);
        assert_eq!(Cursor::new([0x81, 0x00]).get_i16().unwrap(), -1);
        roundtrip(-64, &[0x7F]);
        roundtrip(-63, &[0x7D]);
        roundtrip(-1, &[0x01]);
        roundtrip(0, &[0]);
        roundtrip(1, &[0x02]);
        roundtrip(62, &[0x7C]);
        roundtrip(63, &[0x7E]);
        assert_eq!(Cursor::new([0x80, 0x00]).get_i16().unwrap(), 0);
        roundtrip(64, &[0x80, 0x01]);
        assert_eq!(Cursor::new([0x82, 0x00]).get_i16().unwrap(), 1);
        roundtrip(65, &[0x82, 0x01]);
        roundtrip(8190, &[0xFC, 0x7F]);
        roundtrip(8191, &[0xFE, 0x7F]);
        roundtrip(8192, &[0x80, 0x80, 0x01]);
        roundtrip(8193, &[0x82, 0x80, 0x01]);
        roundtrip(i16::MAX, &[0xFE, 0xFF, 0x03]);
    }

    #[test]
    fn u16_corner_cases() {
        fn roundtrip(v: u16, expected_bytes: &[u8]) {
            let mut w = Cursor::new(vec![]);
            w.put_u16(v).unwrap();
            let bytes = w.into_inner();
            assert_eq!(bytes, expected_bytes);
            let mut r = Cursor::new(expected_bytes);
            let decoded_value = r.get_u16().unwrap();
            assert_eq!(decoded_value, v);
        }

        roundtrip(0, &[0x00]);
        roundtrip(1, &[0x01]);
        roundtrip(126, &[0x7E]);
        roundtrip(127, &[0x7F]);
        roundtrip(128, &[0x80, 0x01]);
        roundtrip(129, &[0x81, 0x01]);
        roundtrip(16382, &[0xFE, 0x7F]);
        roundtrip(16383, &[0xFF, 0x7F]);
        roundtrip(16384, &[0x80, 0x80, 0x01]);
        roundtrip(16385, &[0x81, 0x80, 0x01]);
        roundtrip(65534, &[0xFE, 0xFF, 0x03]);
        roundtrip(65535, &[0xFF, 0xFF, 0x03]);
    }

    #[test]
    fn i32_corner_cases() {
        fn roundtrip(v: i32, expected_bytes: &[u8]) {
            let mut w = Cursor::new(vec![]);
            w.put_i32(v).unwrap();
            let bytes = w.into_inner();
            assert_eq!(
                bytes,
                expected_bytes,
                "for {}, zigzag: {}",
                v,
                zig_zag_encode::encode_i32(v)
            );
            let mut r = Cursor::new(expected_bytes);
            let decoded_value = r.get_i32().unwrap();
            assert_eq!(decoded_value, v);
        }

        roundtrip(
            i32::MIN,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ); // 10 bytes
        roundtrip(
            -1073741825,
            &[0x81, 0x80, 0x80, 0x80, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ); // 10 bytes
        roundtrip(-1073741824, &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]); // 5 bytes
        roundtrip(-134217729, &[0x81, 0x80, 0x80, 0x80, 0x01]); // 5 bytes
        roundtrip(-134217728, &[0xFF, 0xFF, 0xFF, 0x7F]); // 4 bytes
        roundtrip(-1048577, &[0x81, 0x80, 0x80, 0x01]); // 4 bytes
        roundtrip(-1048576, &[0xFF, 0xFF, 0x7F]);
        roundtrip(-8194, &[0x83, 0x80, 0x01]);
        roundtrip(-8193, &[0x81, 0x80, 0x01]);
        roundtrip(-8192, &[0xFF, 0x7F]);
        roundtrip(-8191, &[0xFD, 0x7F]);
        roundtrip(-66, &[0x83, 0x01]);
        roundtrip(-65, &[0x81, 0x01]);
        roundtrip(-64, &[0x7F]);
        roundtrip(-63, &[0x7D]);
        roundtrip(-1, &[0x01]);
        roundtrip(0, &[0]);
        roundtrip(1, &[0x02]);
        roundtrip(62, &[0x7C]);
        roundtrip(63, &[0x7E]);
        roundtrip(64, &[0x80, 0x01]);
        roundtrip(65, &[0x82, 0x01]);
        roundtrip(8190, &[0xFC, 0x7F]);
        roundtrip(8191, &[0xFE, 0x7F]);
        roundtrip(8192, &[0x80, 0x80, 0x01]);
        roundtrip(8193, &[0x82, 0x80, 0x01]);
        roundtrip(1048575, &[0xFE, 0xFF, 0x7F]);
        roundtrip(1048576, &[0x80, 0x80, 0x80, 0x01]); // 4 bytes
        roundtrip(134217727, &[0xFE, 0xFF, 0xFF, 0x7F]); // 4 bytes
        roundtrip(134217728, &[0x80, 0x80, 0x80, 0x80, 0x01]); // 5 bytes
        roundtrip(1073741823, &[0xFE, 0xFF, 0xFF, 0xFF, 0x07]); // 5 bytes
        roundtrip(
            1073741824,
            &[0x80, 0x80, 0x80, 0x80, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ); // 10 bytes
        roundtrip(
            i32::MAX,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ); // 10 bytes
    }

    #[test]
    fn u32_corner_cases() {
        fn roundtrip(v: u32, expected_bytes: &[u8]) {
            let mut w = Cursor::new(vec![]);
            w.put_u32(v).unwrap();
            let bytes = w.into_inner();
            assert_eq!(bytes, expected_bytes, "for {}", v);
            let mut r = Cursor::new(expected_bytes);
            let decoded_value = r.get_u32().unwrap();
            assert_eq!(decoded_value, v);
        }

        roundtrip(0, &[0]);
        roundtrip(126, &[0x7E]);
        roundtrip(127, &[0x7F]);
        roundtrip(128, &[0x80, 0x01]);
        roundtrip(129, &[0x81, 0x01]);
        roundtrip(16383, &[0xFF, 0x7F]);
        roundtrip(16384, &[0x80, 0x80, 0x01]);
        roundtrip(16385, &[0x81, 0x80, 0x01]);
        roundtrip(2097151, &[0xFF, 0xFF, 0x7F]);
        roundtrip(2097152, &[0x80, 0x80, 0x80, 0x01]);
        roundtrip(268435455, &[0xFF, 0xFF, 0xFF, 0x7F]);
        roundtrip(268435456, &[0x80, 0x80, 0x80, 0x80, 0x01]);
        roundtrip(u32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn i64_corner_cases() {
        fn roundtrip(v: i64, expected_bytes: &[u8]) {
            let mut w = Cursor::new(vec![]);
            w.put_i64(v).unwrap();
            let bytes = w.into_inner();
            assert_eq!(bytes, expected_bytes, "for {}", v);
            let mut r = Cursor::new(expected_bytes);
            let decoded_value = r.get_i64().unwrap();
            assert_eq!(decoded_value, v);
        }

        roundtrip(
            i64::MIN,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ); // 10 bytes
        roundtrip(
            i64::MIN / 2 - 1,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 10 bytes
        roundtrip(
            i64::MIN / 2,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 9 bytes
        roundtrip(
            -36028797018963969,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 9 bytes
        roundtrip(
            -36028797018963968,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 8 bytes
        roundtrip(
            -281474976710657,
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 8 bytes
        roundtrip(
            -281474976710656,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 7 bytes
        roundtrip(-2199023255553, &[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]); // 7 bytes
        roundtrip(-2199023255552, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 6 bytes
        roundtrip(-17179869185, &[0x81, 0x80, 0x80, 0x80, 0x80, 0x01]); // 6 bytes
        roundtrip(-17179869184, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 5 bytes
        roundtrip(-1073741824, &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]); // 5 bytes
        roundtrip(-134217729, &[0x81, 0x80, 0x80, 0x80, 0x01]); // 5 bytes
        roundtrip(-134217728, &[0xFF, 0xFF, 0xFF, 0x7F]); // 4 bytes
        roundtrip(-1048577, &[0x81, 0x80, 0x80, 0x01]); // 4 bytes
        roundtrip(-1048576, &[0xFF, 0xFF, 0x7F]);
        roundtrip(-8194, &[0x83, 0x80, 0x01]);
        roundtrip(-8193, &[0x81, 0x80, 0x01]);
        roundtrip(-8192, &[0xFF, 0x7F]);
        roundtrip(-8191, &[0xFD, 0x7F]);
        roundtrip(-66, &[0x83, 0x01]);
        roundtrip(-65, &[0x81, 0x01]);
        roundtrip(-64, &[0x7F]);
        roundtrip(-63, &[0x7D]);
        roundtrip(-1, &[0x01]);
        roundtrip(0, &[0]);
        roundtrip(1, &[0x02]);
        roundtrip(62, &[0x7C]);
        roundtrip(63, &[0x7E]);
        roundtrip(64, &[0x80, 0x01]);
        roundtrip(65, &[0x82, 0x01]);
        roundtrip(8190, &[0xFC, 0x7F]);
        roundtrip(8191, &[0xFE, 0x7F]);
        roundtrip(8192, &[0x80, 0x80, 0x01]);
        roundtrip(8193, &[0x82, 0x80, 0x01]);
        roundtrip(1048575, &[0xFE, 0xFF, 0x7F]);
        roundtrip(1048576, &[0x80, 0x80, 0x80, 0x01]); // 4 bytes
        roundtrip(134217727, &[0xFE, 0xFF, 0xFF, 0x7F]); // 4 bytes
        roundtrip(134217728, &[0x80, 0x80, 0x80, 0x80, 0x01]); // 5 bytes
        roundtrip(1073741823, &[0xFE, 0xFF, 0xFF, 0xFF, 0x07]); // 5 bytes
        roundtrip(17179869183, &[0xFE, 0xFF, 0xFF, 0xFF, 0x7F]); // 5 bytes
        roundtrip(17179869184, &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]); // 6 bytes
        roundtrip(2199023255551, &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 6 bytes
        roundtrip(2199023255552, &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]); // 7 bytes
        roundtrip(281474976710655, &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 7 bytes
        roundtrip(
            281474976710656,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 8 bytes
        roundtrip(
            36028797018963967,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 8 bytes
        roundtrip(
            36028797018963968,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 9 bytes
        roundtrip(
            i64::MAX / 2,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 9 bytes
        roundtrip(
            i64::MAX / 2 + 1,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 10 bytes
        roundtrip(
            i64::MAX,
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ); // 10 bytes
    }

    #[test]
    fn u64_corner_cases() {
        fn roundtrip(v: u64, expected_bytes: &[u8]) {
            let mut w = Cursor::new(vec![]);
            w.put_u64(v).unwrap();
            let bytes = w.into_inner();
            assert_eq!(bytes, expected_bytes, "for {}", v);
            let mut r = Cursor::new(expected_bytes);
            let decoded_value = r.get_u64().unwrap();
            assert_eq!(decoded_value, v);
        }

        roundtrip(0, &[0]);

        roundtrip(126, &[0x7E]);
        roundtrip(127, &[0x7F]);
        roundtrip(128, &[0x80, 0x01]);
        roundtrip(129, &[0x81, 0x01]);
        roundtrip(16383, &[0xFF, 0x7F]);
        roundtrip(16384, &[0x80, 0x80, 0x01]);
        roundtrip(16385, &[0x81, 0x80, 0x01]);
        roundtrip(2097151, &[0xFF, 0xFF, 0x7F]);
        roundtrip(2097152, &[0x80, 0x80, 0x80, 0x01]); // 4 bytes
        roundtrip(268435455, &[0xFF, 0xFF, 0xFF, 0x7F]); // 4 bytes
        roundtrip(268435456, &[0x80, 0x80, 0x80, 0x80, 0x01]); // 5 bytes
        roundtrip(34359738367, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 5 bytes
        roundtrip(34359738368, &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]); // 6 bytes
        roundtrip(4398046511103, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 6 bytes
        roundtrip(4398046511104, &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]); // 7 bytes
        roundtrip(562949953421311, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]); // 7 bytes
        roundtrip(
            562949953421312,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 8 bytes
        roundtrip(
            72057594037927935,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 8 bytes
        roundtrip(
            72057594037927936,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ); // 9 bytes
        roundtrip(
            i64::MAX as u64,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        ); // 10 bytes
           // roundtrip(
           //     i64::MAX as u64 + 1,
           //     &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
           // ); // 10 bytes
    }

    #[cfg(test)]
    proptest! {

        #[test]
        fn u64_check_size_1(v in 0u64..=127u64) {
            prop_assert_eq!(bytes_u64(v).len(), 1);
        }

        #[test]
        fn u64_check_size_2(v in 128u64..=16383u64) {
            prop_assert_eq!(bytes_u64(v).len(), 2);
        }
        #[test]
        fn u64_check_size_3(v in 16384u64..=2097151u64) {
            prop_assert_eq!(bytes_u64(v).len(), 3);
        }
        #[test]
        fn u64_check_size_4(v in 2097152u64..=268435455u64) {
            prop_assert_eq!(bytes_u64(v).len(), 4);
        }

        #[test]
        fn u64_check_size_5(v in 268435456u64..=34359738367u64) {
            prop_assert_eq!(bytes_u64(v).len(), 5);
        }

        #[test]
        fn u64_check_size_6(v in 34359738368u64..=4398046511103u64) {
            prop_assert_eq!(bytes_u64(v).len(), 6);
        }

        #[test]
        fn u64_check_size_7(v in 4398046511104u64..=562949953421311u64) {
            prop_assert_eq!(bytes_u64(v).len(), 7);
        }

        #[test]
        fn u64_check_size_8(v in 562949953421312u64..=72057594037927935u64) {
            prop_assert_eq!(bytes_u64(v).len(), 8);
        }

        #[test]
        fn u64_check_size_9(v in 72057594037927936u64..=i64::MAX as u64) {
            prop_assert_eq!(bytes_u64(v).len(), 9);
        }

        #[test]
        fn i64_check_size_1(v in -64i64..=64i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 1);
        }

        #[test]
        fn i64_check_size_2_part1(v in -8192i64..=-64i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 2);
        }
        #[test]
        fn i64_check_size_2_part2(v in 64i64..=8192i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 2);
        }

        #[test]
        fn i64_check_size_3_part1(v in -1048576i64..=-8192i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 3);
        }
        #[test]
        fn i64_check_size_3_part2(v in 8192i64..=1048576i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 3);
        }

        #[test]
        fn i64_check_size_4_part1(v in -134217728i64..=-1048576i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 4);
        }
        #[test]
        fn i64_check_size_4_part2(v in 1048576i64..=134217728i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 4);
        }

        #[test]
        fn i64_check_size_5_part1(v in -17179869184i64..=-134217728i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 5);
        }
        #[test]
        fn i64_check_size_5_part2(v in 134217728i64..=17179869184i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 5);
        }

        #[test]
        fn i64_check_size_6_part1(v in -2199023255552i64..=-17179869184i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 6);
        }
        #[test]
        fn i64_check_size_6_part2(v in 17179869184i64..=2199023255552i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 6);
        }

        #[test]
        fn i64_check_size_7_part1(v in -281474976710656i64..=-2199023255552i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 7);
        }
        #[test]
        fn i64_check_size_7_part2(v in 2199023255552i64..=281474976710656i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 7);
        }

        #[test]
        fn i64_check_size_8_part1(v in -36028797018963968i64..=-281474976710656i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 8);
        }
        #[test]
        fn i64_check_size_8_part2(v in 281474976710656i64..=36028797018963968i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 8);
        }

        #[test]
        fn i64_check_size_9_part1(v in i64::MIN / 2..=-36028797018963968i64 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 9);
        }
        #[test]
        fn i64_check_size_9_part2(v in 36028797018963968i64..=i64::MAX / 2) {
            prop_assert_eq!(bytes_i64(v).len(), 9);
        }

        #[test]
        fn i64_check_size_10_part1(v in i64::MIN..=i64::MIN / 2 - 1) {
            prop_assert_eq!(bytes_i64(v).len(), 10);
        }
        #[test]
        fn i64_check_size_10_part2(v in i64::MAX / 2 + 1..=i64::MAX) {
            prop_assert_eq!(bytes_i64(v).len(), 10);
        }

        #[test]
        fn u64_roundtrip(i in u64::ANY) {
            let mut w = Cursor::new(vec![]);
            w.put_u64(i).unwrap();
            let mut r = Cursor::new(w.into_inner());
            prop_assert_eq![i, r.get_u64().unwrap()];
        }

        #[test]
        fn i64_roundtrip(i in any::<i64>()) {
            let mut w = Cursor::new(vec![]);
            w.put_i64(i).unwrap();
            let mut r = Cursor::new(w.into_inner());
            prop_assert_eq![i, r.get_i64().unwrap()];
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

        #[test]
        fn prop_bits_roundtrip(bits in collection::vec(any::<bool>(), 0..400)) {
            let mut w = Cursor::new(vec![]);
            w.put_bits(&bits).unwrap();
            let mut r = Cursor::new(w.into_inner());
            prop_assert_eq![bits.clone(), r.get_bits(bits.len()).unwrap()];
        }

        #[test]
        fn prop_short_string_roundtrip(s in ".{1,255}".prop_filter("Filter strings that are too large", |s| s.len() < 256)) {
            let mut w = Cursor::new(vec![]);
            w.put_short_string(&s).unwrap();
            let inner = w.into_inner();
            prop_assert_eq!(inner[0] as usize, s.len());
            prop_assert_eq!(std::str::from_utf8(&inner[1..]), Ok(&*s));
        }

        #[test]
        fn arbitrary_values_list(vals in collection::vec(any::<Val>(), 0..100)) {
            let mut w = Cursor::new(vec![]);
            for val in vals.clone() {
                match val {
                    Val::I8(v) => w.put_i8(v).unwrap(),
                    Val::U8(v) => w.put_u8(v).unwrap(),
                    Val::I16(v) => w.put_i16(v).unwrap(),
                    Val::U16(v) => w.put_u16(v).unwrap(),
                    Val::I32(v) => w.put_i32(v).unwrap(),
                    Val::U32(v) => w.put_u32(v).unwrap(),
                    Val::I64(v) => w.put_i64(v).unwrap(),
                    Val::U64(v) => w.put_u64(v).unwrap(),
                    Val::Bytes(v) => w.write_all(&v).unwrap(),
                    Val::Bits(v) => w.put_bits(&v).unwrap(),
                }

            }
            let mut r = Cursor::new(w.into_inner());
            let mut parsed_vals: Vec<Val> = Vec::new();
            for val in vals.clone() {
                match val {
                    Val::I8(_) => parsed_vals.push(r.get_i8().unwrap().into()),
                    Val::U8(_) => parsed_vals.push(r.get_u8().unwrap().into()),
                    Val::I16(_) => parsed_vals.push(r.get_i16().unwrap().into()),
                    Val::U16(_) => parsed_vals.push(r.get_u16().unwrap().into()),
                    Val::I32(_) => parsed_vals.push(r.get_i32().unwrap().into()),
                    Val::U32(_) => parsed_vals.push(r.get_u32().unwrap().into()),
                    Val::I64(_) => parsed_vals.push(r.get_i64().unwrap().into()),
                    Val::U64(_) => parsed_vals.push(r.get_u64().unwrap().into()),
                    Val::Bytes(bytes) => {
                        let mut buf = vec![0u8; bytes.len()];
                        r.read_exact(&mut buf).unwrap();
                        parsed_vals.push(buf.to_vec().into());
                    },
                    Val::Bits(bits) => parsed_vals.push(r.get_bits(bits.len()).unwrap().into()),
                }
            }
            prop_assert_eq!(parsed_vals, vals);
        }

        #[test]
        fn u16_u32_u64_equivalence(i in any::<u16>()) {
            let expected_bytes = bytes_u16(i);
            prop_assert_eq!(&bytes_u64(i as u64), &expected_bytes);
            prop_assert_eq!(&bytes_u32(i as u32), &expected_bytes);
        }

        #[test]
        fn i16_i32_i64_equivalence(i in any::<i16>()) {
            let expected_bytes = bytes_i16(i);
            prop_assert_eq!(&bytes_i64(i as i64), &expected_bytes);
            prop_assert_eq!(&bytes_i32(i as i32), &expected_bytes);
        }
    }
}
