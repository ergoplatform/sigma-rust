use super::zig_zag_encoding;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

pub trait WriteSigmaVlqExt {
    /// Length of encoded data
    // fn length(&self) -> usize;

    fn put_i8(&mut self, v: i8) -> Result<(), Error> {
        Self::put_u8(self, v as u8)
    }

    fn put_u8(&mut self, v: u8) -> Result<(), Error>;

    fn put_i16(&mut self, v: i16) -> Result<(), Error> {
        Self::put_u32(self, zig_zag_encoding::encode_i32(v as i32))
    }

    fn put_u16(&mut self, v: u16) -> Result<(), Error> {
        Self::put_u64(self, v as u64)
    }

    fn put_i32(&mut self, v: i32) -> Result<(), Error> {
        Self::put_u64(self, zig_zag_encoding::encode_i32(v as i32) as u64)
    }

    fn put_u32(&mut self, v: u32) -> Result<(), Error> {
        Self::put_u64(self, v as u64)
    }

    fn put_i64(&mut self, v: i64) -> Result<(), Error> {
        Self::put_u64(self, zig_zag_encoding::encode_i64(v))
    }

    fn put_u64(&mut self, v: u64) -> Result<(), Error>;

    fn put_slice(&mut self, v: &[u8]) -> Result<(), Error>;

    // fn put_bits(&mut self, _: &[bool]) -> Result<(), Error> {
    //     // implement via put_slice
    //     unimplemented!()
    // }

    // fn put_option<T>(&mut self, v: Option<T>) -> Result<(), Error>;

    fn put_bool(&mut self, v: bool) -> Result<(), Error> {
        Self::put_u8(self, if v { 1 } else { 0 })
    }
}

impl<W: Write> WriteSigmaVlqExt for W {
    fn put_u8(&mut self, v: u8) -> Result<(), Error> {
        self.write_all(&[v]).map_err(Error::Io)
    }

    fn put_u64(&mut self, v: u64) -> Result<(), Error> {
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
        Self::put_slice(self, &buffer[..position])
    }

    fn put_slice(&mut self, v: &[u8]) -> Result<(), Error> {
        self.write_all(v).map_err(Error::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn smoke_test_u8() {
        let mut w = Cursor::new(vec![]);
        w.put_u8(0).unwrap();
        w.put_u8(1).unwrap();
        w.put_u8(255).unwrap();

        assert_eq!(w.into_inner(), vec![0, 1, 255])
    }

    #[test]
    fn smoke_test_slice() {
        let mut w = Cursor::new(vec![]);
        let bytes = vec![0, 2, 255];
        w.put_slice(&bytes).unwrap();

        assert_eq!(w.into_inner(), bytes)
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
    fn test_u64_expected_values() {
        for pair in expected_values() {
            let (bytes, value) = pair;
            let mut w = Cursor::new(vec![]);
            w.put_u64(value).unwrap();
            assert_eq!(w.into_inner(), bytes)
        }
    }
}
