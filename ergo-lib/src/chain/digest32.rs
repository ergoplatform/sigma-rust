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
use std::io;
use thiserror::Error;

/// 32 byte array used in box, transaction ids (hash)
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "Base16EncodedBytes", try_from = "Base16DecodedBytes")
)]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
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

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Digest32 {
    Digest32(sigma_util::hash::blake2b256_hash(bytes))
}

impl From<[u8; Digest32::SIZE]> for Digest32 {
    fn from(bytes: [u8; Digest32::SIZE]) -> Self {
        Digest32(Box::new(bytes))
    }
}

impl Into<Base16EncodedBytes> for Digest32 {
    fn into(self) -> Base16EncodedBytes {
        Base16EncodedBytes::new(self.0.as_ref())
    }
}

impl From<Digest32> for Vec<i8> {
    fn from(v: Digest32) -> Self {
        v.0.to_vec().as_vec_i8()
    }
}

impl TryFrom<Base16DecodedBytes> for Digest32 {
    type Error = Digest32Error;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let arr: [u8; Digest32::SIZE] = bytes.0.as_slice().try_into()?;
        Ok(Digest32(Box::new(arr)))
    }
}

impl Into<String> for Digest32 {
    fn into(self) -> String {
        let bytes: Base16EncodedBytes = self.into();
        bytes.into()
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
#[error("Invalid byte array size ({0})")]
pub struct Digest32Error(std::array::TryFromSliceError);

impl From<std::array::TryFromSliceError> for Digest32Error {
    fn from(err: std::array::TryFromSliceError) -> Self {
        Digest32Error(err)
    }
}
