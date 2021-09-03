use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializeResult;
use ergotree_ir::util::AsVecI8;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Formatter;
use thiserror::Error;

/// N-bytes array in a box. `Digest32` is most type synonym.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "Base16EncodedBytes", try_from = "Base16DecodedBytes")
)]
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Digest<const N: usize>(pub Box<[u8; N]>);

/// 32 byte array used as ID of some value: block, transaction, etc.
/// Usually this is as blake2b hash of serialized form
pub type Digest32 = Digest<32>;

/// AVL tree digest: root hash along with tree height (33 bytes)
pub type ADDigest = Digest<33>;

impl<const N: usize> Digest<N> {
    /// Digest size 32 bytes
    pub const SIZE: usize = N;

    /// All zeros
    pub fn zero() -> Digest<N> {
        Digest(Box::new([0u8; N]))
    }
}

impl<const N: usize> std::fmt::Debug for Digest<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        base16::encode_lower(&(*self.0)).fmt(f)
    }
}

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Digest32 {
    Digest(sigma_util::hash::blake2b256_hash(bytes))
}

impl<const N: usize> From<[u8; N]> for Digest<N> {
    fn from(bytes: [u8; N]) -> Self {
        Digest(Box::new(bytes))
    }
}

impl<const N: usize> From<Digest<N>> for Base16EncodedBytes {
    fn from(v: Digest<N>) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}

impl<const N: usize> From<Digest<N>> for Vec<i8> {
    fn from(v: Digest<N>) -> Self {
        v.0.to_vec().as_vec_i8()
    }
}

impl<const N: usize> From<Digest<N>> for Vec<u8> {
    fn from(v: Digest<N>) -> Self {
        v.0.to_vec()
    }
}

impl<const N: usize> From<Digest<N>> for [u8; N] {
    fn from(v: Digest<N>) -> Self {
        *v.0
    }
}

impl<const N: usize> From<Digest<N>> for String {
    fn from(v: Digest<N>) -> Self {
        let bytes: Base16EncodedBytes = v.into();
        bytes.into()
    }
}

impl<const N: usize> TryFrom<Base16DecodedBytes> for Digest<N> {
    type Error = Digest32Error;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let arr: [u8; N] = bytes.0.as_slice().try_into()?;
        Ok(Digest(Box::new(arr)))
    }
}

impl<const N: usize> TryFrom<String> for Digest<N> {
    type Error = Digest32Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = Base16DecodedBytes::try_from(value)?;
        Digest::<N>::try_from(bytes)
    }
}

impl<const N: usize> SigmaSerializable for Digest<N> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.write_all(self.0.as_ref())?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let mut bytes = [0; N];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes.into()))
    }
}

impl AsRef<[u8]> for Digest32 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
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

#[cfg(test)]
mod tests {
    use super::Digest;
    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::{collection::vec, prelude::*};
    use std::convert::TryInto;

    impl<const N: usize> Arbitrary for Digest<N> {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), Self::SIZE)
                .prop_map(|v| Digest(Box::new(v.try_into().unwrap())))
                .boxed()
        }
    }
}
