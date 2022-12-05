//! Digest types for various sizes

use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use sigma_ser::ScorexParsingError;
use sigma_ser::ScorexSerializable;
use sigma_ser::ScorexSerializeResult;
use sigma_util::AsVecI8;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Formatter;
use thiserror::Error;

/// N-bytes array in a box. `Digest32` is most type synonym.
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        into = "crate::Base16EncodedBytes",
        try_from = "crate::Base16DecodedBytes"
    )
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Digest<const N: usize>(pub [u8; N]);

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
        Digest([0u8; N])
    }

    /// Parse Digest<N> from base64 encoded string
    pub fn from_base64(s: &str) -> Result<Digest<N>, DigestNError> {
        let bytes = base64::decode(s)?;
        let arr: [u8; N] = bytes.as_slice().try_into()?;
        Ok(Digest(arr))
    }
}

impl<const N: usize> std::fmt::Debug for Digest<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        base16::encode_lower(&(self.0)).fmt(f)
    }
}

impl<const N: usize> std::fmt::Display for Digest<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        base16::encode_lower(&(self.0)).fmt(f)
    }
}

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Digest32 {
    Digest(*sigma_util::hash::blake2b256_hash(bytes))
}

impl<const N: usize> From<[u8; N]> for Digest<N> {
    fn from(bytes: [u8; N]) -> Self {
        Digest(bytes)
    }
}

impl<const N: usize> From<Box<[u8; N]>> for Digest<N> {
    fn from(bytes: Box<[u8; N]>) -> Self {
        Digest(*bytes)
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
        v.0
    }
}

impl<const N: usize> From<Digest<N>> for String {
    fn from(v: Digest<N>) -> Self {
        base16::encode_lower(&v.0.as_ref())
    }
}

impl<const N: usize> TryFrom<String> for Digest<N> {
    type Error = DigestNError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = base16::decode(&value)?;
        let arr: [u8; N] = bytes.as_slice().try_into()?;
        Ok(Digest(arr))
    }
}

impl<const N: usize> TryFrom<Vec<u8>> for Digest<N> {
    type Error = DigestNError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes: [u8; N] = value.as_slice().try_into()?;
        Ok(Digest::from(bytes))
    }
}

impl<const N: usize> TryFrom<&[u8]> for Digest<N> {
    type Error = DigestNError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; N] = value.try_into()?;
        Ok(Digest::from(bytes))
    }
}

impl<const N: usize> ScorexSerializable for Digest<N> {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        w.write_all(self.0.as_ref())?;
        Ok(())
    }
    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let mut bytes = [0; N];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }
}

impl AsRef<[u8]> for Digest32 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

/// Invalid byte array size
#[derive(Error, Debug)]
pub enum DigestNError {
    /// error decoding from Base16
    #[error("error decoding from Base16: {0}")]
    Base16DecodingError(#[from] base16::DecodeError),
    /// Invalid byte array size
    #[error("Invalid byte array size ({0})")]
    InvalidSize(#[from] std::array::TryFromSliceError),
    /// error decoding from Base64
    #[error("error decoding from Base64: {0}")]
    Base64DecodingError(#[from] base64::DecodeError),
}

/// Arbitrary
#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub(crate) mod arbitrary {

    use super::Digest;
    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::{collection::vec, prelude::*};
    use std::convert::TryInto;

    impl<const N: usize> Arbitrary for Digest<N> {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), Self::SIZE)
                .prop_map(|v| Digest(v.try_into().unwrap()))
                .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_base64() {
        let s = "KkctSmFOZFJnVWtYcDJzNXY4eS9CP0UoSCtNYlBlU2g=";
        assert!(Digest32::from_base64(s).is_ok());
    }
}
