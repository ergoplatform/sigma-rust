//! Digest types for various sizes

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use base64::Engine;
use core::array::TryFromSliceError;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::fmt::Formatter;
use core::str::FromStr;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use sigma_ser::ScorexParsingError;
use sigma_ser::ScorexSerializable;
use sigma_ser::ScorexSerializeResult;
use sigma_util::AsVecI8;
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

    /// Parse `Digest<N>` from base64 encoded string
    pub fn from_base64(s: &str) -> Result<Digest<N>, DigestNError> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(s)?;
        let arr: [u8; N] = bytes.as_slice().try_into()?;
        Ok(Digest(arr))
    }
}

impl<const N: usize> core::fmt::Debug for Digest<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        base16::encode_lower(&(self.0)).fmt(f)
    }
}

impl<const N: usize> core::fmt::Display for Digest<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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

/// Decode Digest<N> from a base16-encoded string
impl<const N: usize> FromStr for Digest<N> {
    type Err = DigestNError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut arr: [u8; N] = [0; N];
        if s.len() / 2 == N {
            base16::decode_slice(s.as_bytes(), &mut arr)?;
            Ok(Digest(arr))
        } else {
            Err(DigestNError::InvalidSize)
        }
    }
}

// TODO: mark for deprecation
impl<const N: usize> TryFrom<String> for Digest<N> {
    type Error = DigestNError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Digest::from_str(&value)
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
        r.get_bytes_into(&mut bytes)?;
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
    #[cfg(feature = "std")]
    #[error("error decoding from Base16: {0}")]
    Base16DecodingError(#[from] base16::DecodeError),
    /// error decoding from Base16
    #[cfg(not(feature = "std"))]
    #[error("error decoding from Base16")]
    Base16DecodingError,
    /// Invalid byte array size
    #[error("Invalid byte array size")]
    InvalidSize,
    /// error decoding from Base64
    #[cfg(feature = "std")]
    #[error("error decoding from Base64: {0}")]
    Base64DecodingError(#[from] base64::DecodeError),
    /// error decoding from Base64
    #[cfg(not(feature = "std"))]
    #[error("error decoding from Base64")]
    Base64DecodingError,
}

impl From<TryFromSliceError> for DigestNError {
    fn from(_: TryFromSliceError) -> Self {
        DigestNError::InvalidSize
    }
}

/// both base16 and base64 don't implement core::error::Error for their error types yet, so we can't use them in thiserror in no_std contexts
#[cfg(not(feature = "std"))]
impl From<base16::DecodeError> for DigestNError {
    fn from(_: base16::DecodeError) -> Self {
        Self::Base16DecodingError
    }
}

#[cfg(not(feature = "std"))]
impl From<base64::DecodeError> for DigestNError {
    fn from(_: base64::DecodeError) -> Self {
        Self::Base64DecodingError
    }
}

/// Arbitrary
#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub(crate) mod arbitrary {

    use super::Digest;
    use core::convert::TryInto;
    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::{collection::vec, prelude::*};

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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_from_base64() {
        let s = "KkctSmFOZFJnVWtYcDJzNXY4eS9CP0UoSCtNYlBlU2g=";
        assert!(Digest32::from_base64(s).is_ok());
    }
    #[cfg(feature = "arbitrary")]
    mod proptests {
        use crate::Digest;
        use core::str::FromStr;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn base16_roundtrip(digest in any::<Digest<32>>()) {
                assert_eq!(Digest::<32>::from_str(&String::from(digest)).unwrap(), digest);
            }
        }
    }
}
