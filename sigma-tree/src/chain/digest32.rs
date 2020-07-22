use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io;
use thiserror::Error;

///
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "with-serde",
    serde(into = "Base16EncodedBytes", try_from = "Base16DecodedBytes")
)]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Digest32(pub Box<[u8; Digest32::SIZE]>);

impl Digest32 {
    /// Digest size 32 bytes
    pub const SIZE: usize = 32;

    /// All zeros
    pub fn zero() -> Digest32 {
        Digest32(Box::new([0u8; Digest32::SIZE]))
    }
}

pub fn blake2b256_hash(bytes: &[u8]) -> Digest32 {
    // unwrap is safe 32 bytes is a valid hash size (<= 512 && 32 % 8 == 0)
    let mut hasher = VarBlake2b::new(Digest32::SIZE).unwrap();
    hasher.update(bytes);
    let hash = hasher.finalize_boxed();
    // unwrap is safe due to hash size is expected to be Digest32::SIZE
    Digest32(hash.try_into().unwrap())
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
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let mut bytes = [0; Digest32::SIZE];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes.into()))
    }
}

#[derive(Error, Debug)]
#[error("Invalid byte array size ({0})")]
pub struct Digest32Error(std::array::TryFromSliceError);

impl From<std::array::TryFromSliceError> for Digest32Error {
    fn from(err: std::array::TryFromSliceError) -> Self {
        Digest32Error(err)
    }
}
