//! The frame every network message after the handshake travels in.
//!
//! As the reference node writes it (`MessageSerializer`): the network's magic bytes, the message's
//! code, the body's length as a big-endian `i32` and, only when the body is not empty, the first
//! four bytes of the body's Blake2b-256 hash followed by the body.
//!
//! The handshake itself is not framed: frames begin after it, and bytes a peer sent after its
//! handshake in the same read belong to the first frame, so they go into the buffer the decoder
//! reads. What a message code means, and whether an unknown one is an error, is left to the caller:
//! this reads and writes frames, not messages.

use std::convert::TryFrom;
use std::io;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use sigma_util::hash::blake2b256_hash;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

/// Length of a network's magic bytes
pub const MAGIC_LENGTH: usize = 4;
/// Length of the body's checksum
const CHECKSUM_LENGTH: usize = 4;
/// Length of the magic bytes, the message code and the body's length
const HEADER_LENGTH: usize = MAGIC_LENGTH + 5;

/// The longest body the reference node accepts (`MessageConstants.MaxMessageSize`)
pub const MAX_BODY_LENGTH: usize = 16_388_608;

/// A network message as it crosses the wire: its code and its serialized body
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MessageFrame {
    /// Message code
    pub code: u8,
    /// Serialized message, possibly empty
    pub body: Bytes,
}

/// A frame that could not be read or written
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum FrameError {
    /// The underlying stream failed
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    /// The frame begins with another network's magic bytes
    #[error("wrong magic bytes, expected {expected:?}, got {received:?}")]
    WrongMagic {
        /// Magic bytes of the network the codec was made for
        expected: [u8; MAGIC_LENGTH],
        /// Magic bytes the frame began with
        received: [u8; MAGIC_LENGTH],
    },
    /// The frame's length field is negative
    #[error("negative body length {0}")]
    NegativeLength(i32),
    /// The body is longer than the codec's limit
    #[error("body length {length} is above the limit {max}")]
    BodyTooLong {
        /// Length of the body
        length: usize,
        /// The codec's limit
        max: usize,
    },
    /// The checksum is not that of the body
    #[error("wrong checksum")]
    WrongChecksum,
}

/// Encoder/Decoder of [`MessageFrame`]s for one network
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FrameCodec {
    magic: [u8; MAGIC_LENGTH],
    max_body_length: usize,
}

impl FrameCodec {
    /// Codec for the network with the given magic bytes, taking bodies of up to [`MAX_BODY_LENGTH`]
    pub fn new(magic: [u8; MAGIC_LENGTH]) -> Self {
        Self {
            magic,
            max_body_length: MAX_BODY_LENGTH,
        }
    }

    /// The same codec with another limit on a body's length, for reading and for writing. The length
    /// field is an `i32`, so a limit above `i32::MAX` is taken as `i32::MAX`.
    pub fn with_max_body_length(self, max_body_length: usize) -> Self {
        Self {
            max_body_length: max_body_length.min(i32::MAX as usize),
            ..self
        }
    }

    fn too_long(&self, length: usize) -> FrameError {
        FrameError::BodyTooLong {
            length,
            max: self.max_body_length,
        }
    }
}

fn checksum(body: &[u8]) -> [u8; CHECKSUM_LENGTH] {
    let hash = blake2b256_hash(body);
    [hash[0], hash[1], hash[2], hash[3]]
}

impl Decoder for FrameCodec {
    type Item = MessageFrame;
    type Error = FrameError;

    /// The first whole frame in `src`, or `None`, leaving `src` as it was, until one has arrived.
    ///
    /// The header is checked as soon as it is there, before the body it announces: no buffer is
    /// reserved on the length a peer states.
    ///
    /// An error is final for the connection. `src` is left as it was, the bytes that caused the
    /// error included, so there is no frame boundary to resume from: drop the stream.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_LENGTH {
            return Ok(None);
        }
        let mut header = &src[..HEADER_LENGTH];
        let mut received = [0u8; MAGIC_LENGTH];
        header.copy_to_slice(&mut received);
        let code = header.get_u8();
        let length = header.get_i32();

        if received != self.magic {
            return Err(FrameError::WrongMagic {
                expected: self.magic,
                received,
            });
        }
        let length = usize::try_from(length).map_err(|_| FrameError::NegativeLength(length))?;
        if length > self.max_body_length {
            return Err(self.too_long(length));
        }
        if length == 0 {
            src.advance(HEADER_LENGTH);
            return Ok(Some(MessageFrame {
                code,
                body: Bytes::new(),
            }));
        }
        let frame_length = HEADER_LENGTH + CHECKSUM_LENGTH + length;
        if src.len() < frame_length {
            return Ok(None);
        }
        let body_start = HEADER_LENGTH + CHECKSUM_LENGTH;
        if src[HEADER_LENGTH..body_start] != checksum(&src[body_start..frame_length]) {
            return Err(FrameError::WrongChecksum);
        }
        let mut frame = src.split_to(frame_length);
        frame.advance(body_start);
        Ok(Some(MessageFrame {
            code,
            body: frame.freeze(),
        }))
    }
}

impl Encoder<MessageFrame> for FrameCodec {
    type Error = FrameError;

    fn encode(&mut self, item: MessageFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let length = item.body.len();
        if length > self.max_body_length {
            return Err(self.too_long(length));
        }
        let length_field = i32::try_from(length).map_err(|_| self.too_long(length))?;
        dst.reserve(
            HEADER_LENGTH
                + if length > 0 {
                    CHECKSUM_LENGTH + length
                } else {
                    0
                },
        );
        dst.put_slice(&self.magic);
        dst.put_u8(item.code);
        dst.put_i32(length_field);
        if length > 0 {
            dst.put_slice(&checksum(&item.body));
            dst.put_slice(&item.body);
        }
        Ok(())
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const MAINNET: [u8; MAGIC_LENGTH] = [1, 0, 2, 4];
    const TESTNET: [u8; MAGIC_LENGTH] = [2, 3, 2, 3];

    fn from_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn frame(code: u8, body: &[u8]) -> MessageFrame {
        MessageFrame {
            code,
            body: Bytes::copy_from_slice(body),
        }
    }

    fn encode(codec: &mut FrameCodec, frame: MessageFrame) -> Vec<u8> {
        let mut dst = BytesMut::new();
        codec.encode(frame, &mut dst).unwrap();
        dst.to_vec()
    }

    /// A frame with the body `01 02 03` and message code 55 on mainnet, as the reference node's
    /// `MessageSerializer` (v6.0.8) writes it. The checksum, `11c0e79b`, is also what Python's
    /// `hashlib.blake2b(bytes([1, 2, 3]), digest_size=32).digest()[:4]` gives.
    const FRAME_WITH_BODY: &str = "01000204370000000311c0e79b010203";
    /// A frame with an empty body (a peers request, code 1), as the reference node writes it: the
    /// header and nothing after it.
    const FRAME_WITHOUT_BODY: &str = "010002040100000000";

    #[test]
    fn writes_the_reference_bytes() {
        let mut codec = FrameCodec::new(MAINNET);
        assert_eq!(
            encode(&mut codec, frame(55, &[1, 2, 3])),
            from_hex(FRAME_WITH_BODY)
        );
        assert_eq!(
            encode(&mut codec, frame(1, &[])),
            from_hex(FRAME_WITHOUT_BODY)
        );
    }

    #[test]
    fn reads_the_reference_bytes() {
        let mut codec = FrameCodec::new(MAINNET);
        let mut src = BytesMut::from(&from_hex(FRAME_WITH_BODY)[..]);
        src.extend_from_slice(&from_hex(FRAME_WITHOUT_BODY));
        assert_eq!(codec.decode(&mut src).unwrap(), Some(frame(55, &[1, 2, 3])));
        assert_eq!(codec.decode(&mut src).unwrap(), Some(frame(1, &[])));
        assert_eq!(codec.decode(&mut src).unwrap(), None);
        assert!(src.is_empty());
    }

    #[test]
    fn waits_for_a_whole_frame() {
        let bytes = from_hex(FRAME_WITH_BODY);
        let mut codec = FrameCodec::new(MAINNET);
        for n in 0..bytes.len() {
            let mut src = BytesMut::from(&bytes[..n]);
            assert_eq!(codec.decode(&mut src).unwrap(), None);
            assert_eq!(&src[..], &bytes[..n]);
        }
    }

    #[test]
    fn keeps_what_follows_a_frame() {
        let mut bytes = from_hex(FRAME_WITH_BODY);
        bytes.extend_from_slice(&[9, 9, 9]);
        let mut src = BytesMut::from(&bytes[..]);
        let mut codec = FrameCodec::new(MAINNET);
        assert_eq!(codec.decode(&mut src).unwrap(), Some(frame(55, &[1, 2, 3])));
        assert_eq!(&src[..], &[9, 9, 9]);
    }

    #[test]
    fn rejects_another_networks_frame_at_its_header() {
        // the header alone is enough: the body has not arrived
        let mut src = BytesMut::from(&from_hex(FRAME_WITH_BODY)[..HEADER_LENGTH]);
        match FrameCodec::new(TESTNET).decode(&mut src) {
            Err(FrameError::WrongMagic { expected, received }) => {
                assert_eq!(expected, TESTNET);
                assert_eq!(received, MAINNET);
            }
            other => panic!("expected WrongMagic, got {:?}", other),
        }
        // an error leaves the buffer as it was
        assert_eq!(src.len(), HEADER_LENGTH);
    }

    #[test]
    fn rejects_a_negative_length() {
        // length field ffffffff = -1
        let mut src = BytesMut::from(&from_hex("0100020437ffffffff")[..]);
        assert!(matches!(
            FrameCodec::new(MAINNET).decode(&mut src),
            Err(FrameError::NegativeLength(-1))
        ));
        assert_eq!(src.len(), HEADER_LENGTH);
    }

    #[test]
    fn reads_a_body_at_the_limit_and_rejects_one_above_it() {
        let mut codec = FrameCodec::new(MAINNET).with_max_body_length(3);
        let mut src = BytesMut::from(&from_hex(FRAME_WITH_BODY)[..]);
        assert_eq!(codec.decode(&mut src).unwrap(), Some(frame(55, &[1, 2, 3])));

        // a header announcing four bytes
        let mut src = BytesMut::from(&from_hex("010002043700000004")[..]);
        assert!(matches!(
            codec.decode(&mut src),
            Err(FrameError::BodyTooLong { length: 4, max: 3 })
        ));
        assert_eq!(src.len(), HEADER_LENGTH);
        let mut dst = BytesMut::new();
        assert!(matches!(
            codec.encode(frame(55, &[1, 2, 3, 4]), &mut dst),
            Err(FrameError::BodyTooLong { length: 4, max: 3 })
        ));
        assert!(dst.is_empty());
    }

    #[test]
    fn a_limit_above_the_length_fields_range_is_taken_as_its_maximum() {
        assert_eq!(
            FrameCodec::new(MAINNET).with_max_body_length(usize::MAX),
            FrameCodec::new(MAINNET).with_max_body_length(i32::MAX as usize)
        );
    }

    #[test]
    fn the_default_limit_is_the_reference_nodes() {
        // MessageConstants.MaxMessageSize = ModifiersSpec.maxMessageSize (2048576) * 4 * 2;
        // length fields 00fa1200 = 16388608 and 00fa1201 = 16388609
        let mut codec = FrameCodec::new(MAINNET);
        let mut src = BytesMut::from(&from_hex("010002043700fa1200")[..]);
        assert_eq!(codec.decode(&mut src).unwrap(), None);
        let mut src = BytesMut::from(&from_hex("010002043700fa1201")[..]);
        assert!(matches!(
            codec.decode(&mut src),
            Err(FrameError::BodyTooLong { .. })
        ));
    }

    #[test]
    fn rejects_a_checksum_that_is_not_the_bodys() {
        let good = from_hex(FRAME_WITH_BODY);
        // each byte of the checksum and of the body, changed in turn
        for i in HEADER_LENGTH..good.len() {
            let mut bytes = good.clone();
            bytes[i] ^= 1;
            let mut src = BytesMut::from(&bytes[..]);
            assert!(matches!(
                FrameCodec::new(MAINNET).decode(&mut src),
                Err(FrameError::WrongChecksum)
            ));
            // an error leaves the buffer as it was
            assert_eq!(&src[..], &bytes[..]);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn roundtrip(
            magic in any::<[u8; MAGIC_LENGTH]>(),
            frames in proptest::collection::vec(
                (any::<u8>(), proptest::collection::vec(any::<u8>(), 0..300)),
                0..5,
            ),
        ) {
            let mut codec = FrameCodec::new(magic);
            let mut buf = BytesMut::new();
            for (code, body) in &frames {
                codec.encode(frame(*code, body), &mut buf).unwrap();
            }
            for (code, body) in &frames {
                prop_assert_eq!(codec.decode(&mut buf).unwrap(), Some(frame(*code, body)));
            }
            prop_assert_eq!(codec.decode(&mut buf).unwrap(), None);
            prop_assert!(buf.is_empty());
        }
    }
}
