use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::util::AsVecI8;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use sigma_ser::vlq_encode;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Formatter;
use std::io;
use thiserror::Error;

/// 32 byte array used in box, transaction ids (hash)
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "Base16EncodedBytes", try_from = "Base16DecodedBytes")
)]
#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Digest32(pub Box<[u8; Digest32::SIZE]>);

impl Digest32 {
    /// Digest size 32 bytes
    pub const SIZE: usize = sigma_util::DIGEST32_SIZE;

    /// All zeros
    pub fn zero() -> Digest32 {
        Digest32(Box::new([0u8; Digest32::SIZE]))
    }
}

impl std::fmt::Debug for Digest32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        base16::encode_lower(&(*self.0)).fmt(f)
    }
}

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Digest32 {
    Digest32(sigma_util::hash::blake2b256_hash(bytes))
}

impl From<[u8; Digest32::SIZE]> for Digest32 {
    fn from(bytes: [u8; Digest32::SIZE]) -> Self {
        Digest32(Box::new(bytes))
    }
}

impl From<Digest32> for Base16EncodedBytes {
    fn from(v: Digest32) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}

impl From<Digest32> for Vec<i8> {
    fn from(v: Digest32) -> Self {
        v.0.to_vec().as_vec_i8()
    }
}

impl From<Digest32> for Vec<u8> {
    fn from(v: Digest32) -> Self {
        v.0.to_vec()
    }
}

impl From<Digest32> for [u8; Digest32::SIZE] {
    fn from(v: Digest32) -> Self {
        *v.0
    }
}

impl From<Digest32> for String {
    fn from(v: Digest32) -> Self {
        let bytes: Base16EncodedBytes = v.into();
        bytes.into()
    }
}

impl TryFrom<Base16DecodedBytes> for Digest32 {
    type Error = Digest32Error;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let arr: [u8; Digest32::SIZE] = bytes.0.as_slice().try_into()?;
        Ok(Digest32(Box::new(arr)))
    }
}

impl TryFrom<String> for Digest32 {
    type Error = Digest32Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = Base16DecodedBytes::try_from(value)?;
        Digest32::try_from(bytes)
    }
}

impl SigmaSerializable for Digest32 {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_all(self.0.as_ref())?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let mut bytes = [0; Digest32::SIZE];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes.into()))
    }
}

/// Invalid byte array size
#[derive(Error, Debug)]
pub enum Digest32Error {
    /// error decoding from Base16
    #[error("error decoding from Base16: {0}")]
    Base16DecodingError(#[from] base16::DecodeError),
    /// Invalid byte array size
    #[error("Invalid byte array size ({0})")]
    InvalidSize(#[from] std::array::TryFromSliceError),
}
