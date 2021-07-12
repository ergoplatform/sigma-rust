#[cfg(test)]
use proptest::{num::i32, num::i64, prelude::*};

/// Encode a 32-bit value with ZigZag. ZigZag encodes signed integers
/// into values that can be efficiently encoded with VLQ. (Otherwise,
/// negative values must be sign-extended to 64 bits to be varint encoded,
/// thus always taking 10 bytes on the wire.)
/// see <https://developers.google.com/protocol-buffers/docs/encoding#types>
///
/// Although result should be of u32 we need to use u64 due to the signed Int
/// used for result in Scala version
pub fn encode_i32(v: i32) -> u64 {
    // Note:  the right-shift must be arithmetic
    // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedOutputStream.java#L934
    ((v << 1) ^ (v >> 31)) as u64
}

/// Decode a signed value previously ZigZag-encoded with [`encode_i32`]
/// see <https://developers.google.com/protocol-buffers/docs/encoding#types>
pub fn decode_u32(v: u64) -> i32 {
    // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedInputStream.java#L553
    (v as u32 >> 1) as i32 ^ -(v as i32 & 1)
}

/// Encode a 64-bit value with ZigZag. ZigZag encodes signed integers
/// into values that can be efficiently encoded with varint. (Otherwise,
/// negative values must be sign-extended to 64 bits to be varint encoded,
/// thus always taking 10 bytes on the wire.)
/// see <https://developers.google.com/protocol-buffers/docs/encoding#types>
pub fn encode_i64(v: i64) -> u64 {
    // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedOutputStream.java#L949
    ((v << 1) ^ (v >> 63)) as u64
}

/// Decode a signed value previously ZigZag-encoded with [`encode_i64`]
/// see <https://developers.google.com/protocol-buffers/docs/encoding#types>
pub fn decode_u64(v: u64) -> i64 {
    // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/main/java/com/google/protobuf/CodedInputStream.java#L566
    ((v >> 1) ^ (-((v & 1) as i64)) as u64) as i64
}
#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[allow(overflowing_literals)]
    #[test]
    fn test_expected_values() {
        // source: http://github.com/google/protobuf/blob/a7252bf42df8f0841cf3a0c85fdbf1a5172adecb/java/core/src/test/java/com/google/protobuf/CodedOutputStreamTest.java#L281
        assert_eq!(0, encode_i32(0));
        assert_eq!(1, encode_i32(-1));
        assert_eq!(2, encode_i32(1));
        assert_eq!(3, encode_i32(-2));
        assert_eq!(0x7FFF_FFFE, encode_i32(0x3FFF_FFFF));
        assert_eq!(0x7FFF_FFFF, encode_i32(0xC000_0000));
        assert_eq!(0xFFFF_FFFE, encode_i32(0x7FFF_FFFF) as i32);
        assert_eq!(0xFFFF_FFFF, encode_i32(0x8000_0000) as i32);

        assert_eq!(0, encode_i64(0));
        assert_eq!(1, encode_i64(-1));
        assert_eq!(2, encode_i64(1));
        assert_eq!(3, encode_i64(-2));
        assert_eq!(0x0000_0000_7FFF_FFFE, encode_i64(0x0000_0000_3FFF_FFFF));
        assert_eq!(0x0000_0000_7FFF_FFFF, encode_i64(0xFFFF_FFFF_C000_0000));
        assert_eq!(0x0000_0000_FFFF_FFFE, encode_i64(0x0000_0000_7FFF_FFFF));
        assert_eq!(0x0000_0000_FFFF_FFFF, encode_i64(0xFFFF_FFFF_8000_0000));
        assert_eq!(0xFFFF_FFFF_FFFF_FFFE, encode_i64(0x7FFF_FFFF_FFFF_FFFF));
        assert_eq!(0xFFFF_FFFF_FFFF_FFFF, encode_i64(0x8000_0000_0000_0000));
    }

    proptest! {

        #[test]
        fn encode_i32_roundtrip(i in i32::ANY) {
            let dec = decode_u32(encode_i32(i));
            prop_assert_eq![i, dec];
        }

        #[test]
        fn encode_i64_roundtrip(i in i64::ANY) {
            let dec = decode_u64(encode_i64(i));
            prop_assert_eq![i, dec];
        }

    }
}
