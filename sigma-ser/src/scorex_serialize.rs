use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{string::String, vec};
use core::convert::TryInto;
use core3::io;
use core3::io::Cursor;

use crate::vlq_encode;
use crate::vlq_encode::*;
use bounded_vec::{BoundedVec, BoundedVecOutOfBounds};
use thiserror::Error;

/// Ways serialization might fail
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum ScorexSerializationError {
    /// IO fail (EOF, etc.)
    #[error("IO error: {0}")]
    Io(String),
    /// Serialization not yet implemented
    #[error("serialization not yet implemented: {0}")]
    NotImplementedYet(&'static str),
    /// Serialization not supported
    #[error("serialization not supported: {0}")]
    NotSupported(&'static str),
    /// Integer type conversion failed
    #[error("Bounds check error: {0}")]
    TryFrom(#[from] core::num::TryFromIntError),
    /// Misc error
    #[error("error: {0}")]
    Misc(&'static str),
}

impl From<io::Error> for ScorexSerializationError {
    fn from(error: io::Error) -> Self {
        ScorexSerializationError::Io(error.to_string())
    }
}

#[cfg(feature = "std")]
impl From<ScorexSerializationError> for io::Error {
    fn from(e: ScorexSerializationError) -> Self {
        io::Error::new(io::ErrorKind::InvalidInput, e.to_string())
    }
}

#[cfg(not(feature = "std"))]
impl From<ScorexSerializationError> for io::Error {
    fn from(_e: ScorexSerializationError) -> Self {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Error messages are only supported on std target",
        )
    }
}

/// Ways parsing might fail
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum ScorexParsingError {
    /// Invalid op code
    #[error("invalid op code: {0}")]
    InvalidOpCode(u8),
    /// Lacking support for the op
    #[error("not implemented op error")]
    NotImplementedOpCode(String),
    /// Failed to parse type
    #[error("type parsing error, invalid type code: {0}({0:#04X})")]
    InvalidTypeCode(u8),
    /// Failed to decode VLQ
    #[error("vlq encode error: {0}")]
    VlqEncode(#[from] vlq_encode::VlqEncodingError),
    /// IO fail (EOF, etc.)
    #[error("IO error: {0}")]
    Io(String),
    /// Misc fail
    #[error("misc error")]
    Misc(String),
    /// Feature not yet implemented
    #[error("parsing not yet implemented: {0}")]
    NotImplementedYet(String),
    /// Value out of bounds
    #[error("Value out of bounds: {0}")]
    ValueOutOfBounds(String),
    /// Tuple items out of bounds
    #[error("Tuple items out of bounds: {0}")]
    TupleItemsOutOfBounds(usize),
    /// Feature not supported
    #[error("parsing not supported: {0}")]
    NotSupported(&'static str),
    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(#[from] ScorexSerializationError),
    /// Invalid item quantity for BoundedVec
    #[error("Invalid item quantity for BoundedVec: {0}")]
    BoundedVecOutOfBounds(#[from] BoundedVecOutOfBounds),
    /// Failed to convert integer type
    #[error("Bounds check error: {0}")]
    TryFrom(#[from] core::num::TryFromIntError),
}

impl From<io::Error> for ScorexParsingError {
    fn from(error: io::Error) -> Self {
        ScorexParsingError::Io(error.to_string())
    }
}

impl From<&io::Error> for ScorexParsingError {
    fn from(error: &io::Error) -> Self {
        ScorexParsingError::Io(error.to_string())
    }
}

/// Result type for [`ScorexSerializable::scorex_serialize`]
pub type ScorexSerializeResult = Result<(), ScorexSerializationError>;

/// Scorex Serializable Trait.
pub trait ScorexSerializable: Sized {
    /// Write `self` to the `writer`
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult;
    /// parse `self` from `reader`
    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError>;

    /// Serialize a ScorexSerializable value into bytes
    fn scorex_serialize_bytes(&self) -> Result<Vec<u8>, ScorexSerializationError> {
        let mut w = vec![];
        self.scorex_serialize(&mut w)?;
        Ok(w)
    }
    /// Parse `self` from the bytes
    fn scorex_parse_bytes(bytes: &[u8]) -> Result<Self, ScorexParsingError> {
        Self::scorex_parse(&mut Cursor::new(bytes))
    }
}

impl<T: ScorexSerializable> ScorexSerializable for Vec<T> {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        w.put_u32(self.len() as u32)?;
        self.iter().try_for_each(|i| i.scorex_serialize(w))
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let items_count = r.get_u32()?;
        // The count is taken from untrusted input: pre-allocating it verbatim
        // lets a few input bytes force an allocation vastly larger than the
        // input itself, aborting the process (an allocation failure is not a
        // recoverable parse error). Cap the pre-allocation; the buffer still
        // grows to the real item count as items are parsed.
        let mut items = Vec::with_capacity(items_count.min(1024) as usize);
        for _ in 0..items_count {
            items.push(T::scorex_parse(r)?);
        }
        Ok(items)
    }
}

/// Read exactly `len` bytes from `r`, growing the buffer only as bytes
/// actually arrive. Reading into a `vec![0; len]` buffer instead allocates the
/// full claimed length up front, so a length prefix taken from untrusted
/// input can force an allocation vastly larger than the input itself and
/// abort the process. Errors exactly like `read_exact` when the stream ends
/// before `len` bytes have been read.
pub fn scorex_parse_bytes_exact<R: ReadSigmaVlqExt>(
    r: &mut R,
    len: usize,
) -> Result<Vec<u8>, ScorexParsingError> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    while buf.len() < len {
        let want = core::cmp::min(chunk.len(), len - buf.len());
        match r.read(&mut chunk[..want]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                )
                .into())
            }
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(buf)
}

impl<T: ScorexSerializable, const L: usize, const U: usize> ScorexSerializable
    for BoundedVec<T, L, U>
{
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        self.as_vec().scorex_serialize(w)
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        Ok(Vec::<T>::scorex_parse(r)?.try_into()?)
    }
}

/// Corresponds to `VLQ(UInt)` format from `ErgoTree` spec.
impl ScorexSerializable for u32 {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        w.put_u32(*self)?;
        Ok(())
    }
    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let v = r.get_u32()?;
        Ok(v)
    }
}

impl<T: ScorexSerializable> ScorexSerializable for Option<Box<T>> {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        match self {
            Some(v) => {
                w.put_u8(1)?;
                v.scorex_serialize(w)
            }
            None => Ok(w.put_u8(0)?),
        }
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let tag = r.get_u8()?;
        Ok(if tag != 0 {
            Some(T::scorex_parse(r)?.into())
        } else {
            None
        })
    }
}

/// serialization roundtrip
#[allow(clippy::expect_used)]
pub fn scorex_serialize_roundtrip<T: ScorexSerializable>(v: &T) -> T {
    let mut data = Vec::new();
    v.scorex_serialize(&mut data).expect("serialization failed");
    let reader = &mut Cursor::new(&data[..]);
    T::scorex_parse(reader).expect("parse failed")
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[allow(clippy::panic)]
mod test {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    /// VLQ (unsigned LEB128) encoding of `u32::MAX`.
    const VLQ_U32_MAX: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, 0x0F];

    #[test]
    fn vec_parse_huge_count_is_error_not_huge_allocation() {
        // A count prefix of u32::MAX with no items following must be a parse
        // error; sizing the buffer by the count aborted the process with a
        // ~17 GB allocation failure (for u32 items) before this fix.
        assert!(Vec::<u32>::scorex_parse_bytes(&VLQ_U32_MAX).is_err());
    }

    #[test]
    fn bytes_exact_reads_exact_bytes_and_errors_on_short_stream() {
        let payload: [u8; 4] = [1, 2, 3, 4];
        let mut reader = Cursor::new(&payload[..]);
        assert_eq!(
            scorex_parse_bytes_exact(&mut reader, 4).unwrap(),
            payload.to_vec()
        );

        let mut reader = Cursor::new(&payload[..]);
        // Same error as read_exact on a short stream.
        assert_eq!(
            scorex_parse_bytes_exact(&mut reader, 5),
            Err(ScorexParsingError::Io(
                "failed to fill whole buffer".to_string()
            ))
        );

        let mut reader = Cursor::new(&payload[..]);
        assert_eq!(
            scorex_parse_bytes_exact(&mut reader, 0).unwrap(),
            Vec::<u8>::new()
        );
    }

    proptest! {
        #[test]
        fn u32_roundtrip(val in any::<u32>()) {
            assert_eq!(scorex_serialize_roundtrip(&val), val);
        }

        #[test]
        fn vec_roundtrip(val in vec(any::<u32>(), 0..255)) {
            assert_eq!(scorex_serialize_roundtrip(&val), val);
        }

        #[test]
        fn box_roundtrip(val in any::<Option<Box<u32>>>()) {
            assert_eq!(scorex_serialize_roundtrip(&val), val);
        }
    }
}
