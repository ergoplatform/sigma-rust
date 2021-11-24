use std::convert::TryInto;
use std::io;

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
}

impl From<io::Error> for ScorexSerializationError {
    fn from(error: io::Error) -> Self {
        ScorexSerializationError::Io(error.to_string())
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
    #[error("type parsing error")]
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
    fn scorex_serialize_bytes(&self) -> Result<Vec<u8>, ScorexParsingError> {
        let mut w = vec![];
        self.scorex_serialize(&mut w)?;
        Ok(w)
    }
    /// Parse `self` from the bytes
    fn scorex_parse_bytes(mut bytes: &[u8]) -> Result<Self, ScorexParsingError> {
        Self::scorex_parse(&mut bytes)
    }
}

impl<T: ScorexSerializable> ScorexSerializable for Vec<T> {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        w.put_u32(self.len() as u32)?;
        self.iter().try_for_each(|i| i.scorex_serialize(w))
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let items_count = r.get_u32()?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(T::scorex_parse(r)?);
        }
        Ok(items)
    }
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
    let reader = &mut &data[..];
    T::scorex_parse(reader).expect("parse failed")
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[allow(clippy::panic)]
mod test {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

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
