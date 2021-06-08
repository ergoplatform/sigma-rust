//! Serialization of Ergo types
use crate::mir::expr::InvalidArgumentError;
use crate::mir::val_def::ValId;
use crate::types::type_unify::TypeUnificationError;

use super::{
    constant_store::ConstantStore,
    sigma_byte_reader::{SigmaByteRead, SigmaByteReader},
    sigma_byte_writer::{SigmaByteWrite, SigmaByteWriter},
};
use crate::serialization::types::TypeCode;
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
pub enum SerializationError {
    /// Invalid op code
    #[error("invalid op code: {0}")]
    InvalidOpCode(u8),
    /// Lacking support for the op
    #[error("not implemented op error")]
    NotImplementedOpCode(String),
    /// Failed to parse type
    #[error("type parsing error")]
    InvalidTypePrefix,
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
    #[error("feature not yet implemented: {0}")]
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
    InvalidArgument(InvalidArgumentError),
    /// Unknown method ID for given type code
    #[error("No method id {0:?} found in type companion with type id {1:?} ")]
    UnknownMethodId(MethodId, TypeCode),
}

impl From<io::Error> for SerializationError {
    fn from(error: io::Error) -> Self {
        SerializationError::Io(error.to_string())
    }
}

impl From<&io::Error> for SerializationError {
    fn from(error: &io::Error) -> Self {
        SerializationError::Io(error.to_string())
    }
}

impl From<InvalidArgumentError> for SerializationError {
    fn from(e: InvalidArgumentError) -> Self {
        SerializationError::InvalidArgument(e)
    }
}

impl From<BoundedVecOutOfBounds> for SerializationError {
    fn from(e: BoundedVecOutOfBounds) -> Self {
        SerializationError::ValueOutOfBounds(format!("{:?}", e))
    }
}

impl From<TypeUnificationError> for SerializationError {
    fn from(e: TypeUnificationError) -> Self {
        SerializationError::Misc(format!("{:?}", e))
    }
}

/// Consensus-critical serialization for Ergo
pub trait SigmaSerializable: Sized {
    /// Write `self` to the given `writer`.
    /// This function has a `sigma_` prefix to alert the reader that the
    /// serialization in use is consensus-critical serialization    
    /// Notice that the error type is [`std::io::Error`]; this indicates that
    /// serialization MUST be infallible up to errors in the underlying writer.
    /// In other words, any type implementing `SigmaSerializable`
    /// must make illegal states unrepresentable.
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error>;

    /// Try to read `self` from the given `reader`.
    /// `sigma-` prefix to alert the reader that the serialization in use
    /// is consensus-critical
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError>;

    /// Serialize any SigmaSerializable value into bytes
    fn sigma_serialize_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        #[allow(clippy::expect_used)]
        self.sigma_serialize(&mut w)
            // since serialization may fail only for underlying IO errors it's ok to force unwrap
            .expect("serialization failed");
        data
    }

    /// Parse `self` from the bytes
    fn sigma_parse_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let cursor = Cursor::new(bytes);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        Self::sigma_parse(&mut sr)
    }
}

impl<T: SigmaSerializable> SigmaSerializable for Vec<T> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u32(self.len() as u32)?;
        self.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
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
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u32(self.len() as u32)?;
        self.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let items_count = r.get_u32()?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(T::sigma_parse(r)?);
        }
        Ok(items.try_into()?)
    }
}

impl<T: SigmaSerializable> SigmaSerializable for Option<Box<T>> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        match self {
            Some(v) => {
                w.put_u8(1)?;
                v.sigma_serialize(w)
            }
            None => w.put_u8(0),
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
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
