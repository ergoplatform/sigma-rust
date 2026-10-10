//! Serialization of Ergo types
use crate::chain::ergo_box::RegisterValueError;
use crate::ergo_tree::{ErgoTreeHeaderError, ErgoTreeVersion};
use crate::mir::val_def::ValId;
use crate::mir::{constant::TryExtractFromError, expr::InvalidArgumentError};
use crate::types::type_unify::TypeUnificationError;

use super::{
    constant_store::ConstantStore,
    sigma_byte_reader::{SigmaByteRead, SigmaByteReader},
    sigma_byte_writer::{SigmaByteWrite, SigmaByteWriter},
};
use crate::types::smethod::MethodId;
use alloc::boxed::Box;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use bounded_vec::BoundedVec;
use bounded_vec::BoundedVecOutOfBounds;
use core::convert::TryInto;
use core3::io;
use io::Cursor;
use sigma_ser::{vlq_encode, ScorexParsingError, ScorexSerializationError};
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
    UnexpectedValue(#[from] TryExtractFromError),
    /// Serialization not supported
    #[error("serialization not supported: {0}")]
    NotSupported(String),
    /// Scorex serialization error
    #[error("Scorex serialization error: {0}")]
    ScorexSerializationError(#[from] ScorexSerializationError),
}

impl From<io::Error> for SigmaSerializationError {
    fn from(error: io::Error) -> Self {
        SigmaSerializationError::Io(error.to_string())
    }
}

/// Ways parsing might fail
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum SigmaParsingError {
    /// Invalid op code
    #[error("invalid op code: {0}")]
    InvalidOpCode(u8),
    /// Lacking support for the op
    #[error("not implemented op error: {0}")]
    NotImplementedOpCode(String),
    /// Failed to parse type
    #[error("type parsing error, invalid type code: {0}({0:#04X})")]
    InvalidTypeCode(u8),
    /// V6 type error
    #[error("Can't use v6 types (UnsignedBigInt, Header, Option) in ContextExtension/Registers ")]
    V6TypeError,
    /// Failed to decode VLQ
    #[error("vlq encode error: {0}")]
    VlqEncode(#[from] vlq_encode::VlqEncodingError),
    /// IO fail (EOF, etc.)
    #[error("IO error: {0}")]
    Io(String),
    /// Misc fail
    #[error("misc error: {0}")]
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
    /// Scorex parsing error
    #[error("Scorex parsing error: {0}")]
    ScorexParsingError(#[from] ScorexParsingError),
    /// ErgoTreeHeaderError
    #[error("ErgoTreeHeaderError: {0}")]
    ErgoTreeHeaderError(#[from] ErgoTreeHeaderError),
    /// Invalid register value
    #[error("Invalid register value: {0}")]
    InvalidRegisterValue(#[from] RegisterValueError),
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
        w.with_tree_version(ErgoTreeVersion::MAX_SCRIPT_VERSION, |w| {
            self.sigma_serialize(w)
        })?;
        Ok(data)
    }

    /// Parse `self` from the bytes
    fn sigma_parse_bytes(bytes: &[u8]) -> Result<Self, SigmaParsingError> {
        let cursor = Cursor::new(bytes);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        // Set version to max for convenience when parsing new types like UnsignedBigInt from bytes/base16
        sr.with_tree_version(ErgoTreeVersion::MAX_SCRIPT_VERSION, |sr| {
            Self::sigma_parse(sr)
        })
    }
}

impl<T: SigmaSerializable> SigmaSerializable for Vec<T> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u32(self.len() as u32)?;
        self.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let items_count = r.get_u32()?;
        // The count is taken from untrusted input: pre-allocating it verbatim
        // lets a few input bytes force an allocation vastly larger than the
        // input itself, aborting the process (an allocation failure is not a
        // recoverable parse error). Cap the pre-allocation; the buffer still
        // grows to the real item count as items are parsed.
        let mut items = Vec::with_capacity(items_count.min(1024) as usize);
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

/// Read exactly `len` bytes from `r`, growing the buffer only as bytes
/// actually arrive. Reading into a `vec![0; len]` buffer instead allocates the
/// full claimed length up front, so a length prefix taken from untrusted
/// input can force an allocation vastly larger than the input itself and
/// abort the process. Errors exactly like `read_exact` when the stream ends
/// before `len` bytes have been read.
pub(crate) fn sigma_parse_bytes_exact<R: SigmaByteRead>(
    r: &mut R,
    len: usize,
) -> Result<Vec<u8>, SigmaParsingError> {
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

/// Perform versioned serialization
pub fn sigma_serialize_roundtrip_versioned<T: SigmaSerializable>(
    v: &T,
    tree_version: ErgoTreeVersion,
) -> Result<T, Box<dyn core::error::Error>> {
    let mut data = Vec::new();
    let mut w = SigmaByteWriter::new(&mut data, None);
    w.with_tree_version(tree_version, |w| v.sigma_serialize(w))?;
    let cursor = Cursor::new(&mut data[..]);
    let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
    sr.with_tree_version(tree_version, T::sigma_parse)
        .map_err(From::from)
}

/// Perform serialization roundtrip for a feature that's only supported after `since`
#[allow(clippy::expect_used)]
pub fn roundtrip_new_feature<T: SigmaSerializable + core::fmt::Debug + PartialEq>(
    v: &T,
    since: ErgoTreeVersion,
) {
    for version in 0u8..since.into() {
        assert!(sigma_serialize_roundtrip_versioned(v, version.into()).is_err());
    }
    for version in u8::from(since)..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into() {
        assert_eq!(
            *v,
            sigma_serialize_roundtrip_versioned(v, version.into()).expect("roundtrip failed")
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::sigma_and::SigmaAnd;
    use crate::serialization::data::DataSerializer;
    use crate::types::stype::SType;

    /// VLQ (unsigned LEB128) encoding of `u32::MAX`.
    const VLQ_U32_MAX: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, 0x0F];

    #[test]
    fn vec_parse_huge_count_is_error_not_huge_allocation() {
        // Regression for #944: SigmaAnd's items go through Vec::<Expr>::
        // sigma_parse. A count prefix of u32::MAX with no items following
        // must be a parse error; sizing the buffer by the count aborted the
        // process with an ~859 GB allocation failure before this fix.
        assert!(SigmaAnd::sigma_parse_bytes(&VLQ_U32_MAX).is_err());
    }

    #[test]
    fn string_parse_huge_len_is_error_not_huge_allocation() {
        // Regression for #944: a String literal whose length prefix claims
        // u32::MAX bytes with nothing following must be a parse error.
        let cursor = Cursor::new(&VLQ_U32_MAX[..]);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        assert!(DataSerializer::sigma_parse(&SType::SString, &mut sr).is_err());
    }

    #[test]
    fn bytes_exact_reads_exact_bytes_and_errors_on_short_stream() {
        let payload: [u8; 4] = [1, 2, 3, 4];
        let cursor = Cursor::new(&payload[..]);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        assert_eq!(
            sigma_parse_bytes_exact(&mut sr, 4).unwrap(),
            payload.to_vec()
        );

        let cursor = Cursor::new(&payload[..]);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        // Same error as read_exact on a short stream.
        assert_eq!(
            sigma_parse_bytes_exact(&mut sr, 5),
            Err(SigmaParsingError::Io(
                "failed to fill whole buffer".to_string()
            ))
        );

        let cursor = Cursor::new(&payload[..]);
        let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
        assert_eq!(
            sigma_parse_bytes_exact(&mut sr, 0).unwrap(),
            Vec::<u8>::new()
        );
    }
}
