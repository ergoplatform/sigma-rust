//! Serialization of Ergo types
use crate::mir::val_def::ValId;
use crate::mir::{constant::TryExtractFromError, expr::InvalidArgumentError};
use crate::types::type_unify::TypeUnificationError;

use super::{
    constant_store::ConstantStore,
    sigma_byte_reader::{SigmaByteRead, SigmaByteReader},
    sigma_byte_writer::{SigmaByteWrite, SigmaByteWriter},
};
use crate::types::smethod::MethodId;
use bounded_vec::BoundedVec;
use bounded_vec::BoundedVecOutOfBounds;
use io::Cursor;
use sigma_ser::vlq_encode;
use std::convert::TryInto;
use std::io;
use thiserror::Error;

/// Ways serialization might fail
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum SigmaSerializationError {
    /// IO fail (EOF, etc.)
    #[error("IO error: {0}")]
    Io(String),
    /// Serialization not yet implemented
    #[error("serialization not yet implemented: {0}")]
    NotImplementedYet(&'static str),
    /// Unexpected value type
    #[error("Unexpected value: {0:?}")]
    UnexpectedValue(TryExtractFromError),
    /// Serialization not supported
    #[error("serialization not supported: {0}")]
    NotSupported(&'static str),
}

impl From<io::Error> for SigmaSerializationError {
    fn from(error: io::Error) -> Self {
        SigmaSerializationError::Io(error.to_string())
    }
}

impl From<TryExtractFromError> for SigmaSerializationError {
    fn from(e: TryExtractFromError) -> Self {
        SigmaSerializationError::UnexpectedValue(e)
    }
}

/// Ways parsing might fail
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum SigmaParsingError {
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
    /// Constant with given index not found in constant store
    #[error("Constant with index {0} not found in constant store")]
    ConstantForPlaceholderNotFound(u32),
    /// Value out of bounds
    #[error("Value out of bounds: {0}")]
    ValueOutOfBounds(String),
    /// Tuple items out of bounds
    #[error("Tuple items out of bounds: {0}")]
    TupleItemsOutOfBounds(usize),
    /// ValDef type for a given index not found in ValDefTypeStore store
    #[error("ValDef type for an index {0:?} not found in ValDefTypeStore store")]
    ValDefIdNotFound(ValId),
    /// Invalid argument on node creation
    #[error("Invalid argument: {0:?}")]
    InvalidArgument(#[from] InvalidArgumentError),
    /// Unknown method ID for given type code
    #[error("No method id {0:?} found in type companion with type id {1:?} ")]
    UnknownMethodId(MethodId, u8),
    /// Feature not supported
    #[error("parsing not supported: {0}")]
    NotSupported(&'static str),
    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(#[from] SigmaSerializationError),
    /// Invalid item quantity for BoundedVec
    #[error("Invalid item quantity for BoundedVec: {0}")]
    BoundedVecOutOfBounds(#[from] BoundedVecOutOfBounds),
}

impl From<io::Error> for SigmaParsingError {
    fn from(error: io::Error) -> Self {
        SigmaParsingError::Io(error.to_string())
    }
}

impl From<&io::Error> for SigmaParsingError {
    fn from(error: &io::Error) -> Self {
        SigmaParsingError::Io(error.to_string())
    }
}

impl From<TypeUnificationError> for SigmaParsingError {
    fn from(e: TypeUnificationError) -> Self {
        SigmaParsingError::Misc(format!("{:?}", e))
    }
}

/// Result type for [`SigmaSerializable::sigma_serialize`]
pub type SigmaSerializeResult = Result<(), SigmaSerializationError>;

/// Consensus-critical serialization for Ergo
pub trait SigmaSerializable: Sized {
    /// Write `self` to the given `writer`.
    /// This function has a `sigma_` prefix to alert the reader that the
    /// serialization in use is consensus-critical serialization    
    // fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult;
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult;

    /// Try to read `self` from the given `reader`.
    /// `sigma-` prefix to alert the reader that the serialization in use
    /// is consensus-critical
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError>;

    /// Serialize any SigmaSerializable value into bytes
    fn sigma_serialize_bytes(&self) -> Result<Vec<u8>, SigmaSerializationError> {
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        self.sigma_serialize(&mut w)?;
        Ok(data)
    }

    /// Parse `self` from the bytes
    fn sigma_parse_bytes(bytes: &[u8]) -> Result<Self, SigmaParsingError> {
        let cursor = Cursor::new(bytes);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        Self::sigma_parse(&mut sr)
    }
}

impl<T: SigmaSerializable> SigmaSerializable for Vec<T> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u32(self.len() as u32)?;
        self.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let items_count = r.get_u32()?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(T::sigma_parse(r)?);
        }
        Ok(items)
    }
}

impl<T: SigmaSerializable, const L: usize, const U: usize> SigmaSerializable
    for BoundedVec<T, L, U>
{
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.as_vec().sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        Ok(Vec::<T>::sigma_parse(r)?.try_into()?)
    }
}

/// Corresponds to `VLQ(UInt)` format from `ErgoTree` spec.
impl SigmaSerializable for u32 {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u32(*self)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let v = r.get_u32()?;
        Ok(v)
    }
}

impl<T: SigmaSerializable> SigmaSerializable for Option<Box<T>> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        match self {
            Some(v) => {
                w.put_u8(1)?;
                v.sigma_serialize(w)
            }
            None => Ok(w.put_u8(0)?),
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let tag = r.get_u8()?;
        Ok(if tag != 0 {
            Some(T::sigma_parse(r)?.into())
        } else {
            None
        })
    }
}

/// serialization roundtrip
#[allow(clippy::expect_used)]
pub fn sigma_serialize_roundtrip<T: SigmaSerializable>(v: &T) -> T {
    let mut data = Vec::new();
    let mut w = SigmaByteWriter::new(&mut data, None);
    v.sigma_serialize(&mut w).expect("serialization failed");
    let cursor = Cursor::new(&mut data[..]);
    let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
    T::sigma_parse(&mut sr).expect("parse failed")
}
