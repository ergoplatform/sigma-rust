use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use ergotree_ir::util::AsVecI8;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use thiserror::Error;

///
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "Base16EncodedBytes", try_from = "Base16DecodedBytes")
)]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ADDigest(pub Box<[u8; Self::SIZE]>);

impl ADDigest {
    /// Digest size 33 bytes
    pub const SIZE: usize = 33;
}

impl From<[u8; ADDigest::SIZE]> for ADDigest {
    fn from(bytes: [u8; ADDigest::SIZE]) -> Self {
        ADDigest(Box::new(bytes))
    }
}

impl From<ADDigest> for Base16EncodedBytes {
    fn from(v: ADDigest) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}

impl From<ADDigest> for Vec<i8> {
    fn from(v: ADDigest) -> Self {
        v.0.to_vec().as_vec_i8()
    }
}

impl From<ADDigest> for Vec<u8> {
    fn from(v: ADDigest) -> Self {
        v.0.to_vec()
    }
}

impl From<ADDigest> for [u8; ADDigest::SIZE] {
    fn from(v: ADDigest) -> Self {
        *(v.0)
    }
}

impl From<ADDigest> for String {
    fn from(v: ADDigest) -> Self {
        let bytes: Base16EncodedBytes = v.into();
        bytes.into()
    }
}

impl TryFrom<Base16DecodedBytes> for ADDigest {
    type Error = ADDigestError;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let arr: [u8; ADDigest::SIZE] = bytes.0.as_slice().try_into()?;
        Ok(ADDigest(Box::new(arr)))
    }
}

impl TryFrom<String> for ADDigest {
    type Error = ADDigestError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = Base16DecodedBytes::try_from(value)?;
        ADDigest::try_from(bytes)
    }
}

/// Invalid byte array size
#[derive(Error, Debug)]
pub enum ADDigestError {
    /// error decoding from Base16
    #[error("error decoding from Base16: {0}")]
    Base16DecodingError(#[from] base16::DecodeError),
    /// Invalid byte array size
    #[error("Invalid byte array size ({0})")]
    InvalidSize(#[from] std::array::TryFromSliceError),
}

#[cfg(test)]
mod tests {
    use super::ADDigest;
    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::{collection::vec, prelude::*};
    use std::convert::TryInto;

    impl Arbitrary for ADDigest {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), Self::SIZE)
                .prop_map(|v| ADDigest(Box::new(v.try_into().unwrap())))
                .boxed()
        }
    }
}
