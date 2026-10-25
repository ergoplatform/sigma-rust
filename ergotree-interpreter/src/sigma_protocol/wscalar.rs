//! Wrapper for Scalar
//! mainly for Arbitrary impl and JSON encoding

use core::array::TryFromSliceError;
use core::convert::TryFrom;
use core::fmt::Formatter;

use derive_more::From;
use derive_more::Into;
use elliptic_curve::PrimeField;
use ergo_chain_types::Base16DecodedBytes;
use ergo_chain_types::Base16EncodedBytes;
use k256::elliptic_curve::ops::Reduce;
use k256::FieldBytes;
use k256::Scalar;

use super::challenge::Challenge;
use super::GroupSizedBytes;
use super::SOUNDNESS_BYTES;

#[derive(PartialEq, Eq, From, Into, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        try_from = "ergo_chain_types::Base16DecodedBytes",
        into = "ergo_chain_types::Base16EncodedBytes"
    )
)]
/// Wrapper for Scalar mainly for Arbitrary impl and JSON encoding
pub struct Wscalar(Scalar);

impl Wscalar {
    /// Scalar(secret key) size in bytes
    pub const SIZE_BYTES: usize = 32;

    /// Returns a reference to underlying Scalar
    pub fn as_scalar_ref(&self) -> &Scalar {
        &self.0
    }

    /// Attempts to parse the given byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_bytes(bytes: &[u8; Self::SIZE_BYTES]) -> Option<Self> {
        k256::Scalar::from_repr((*bytes).into())
            .map(Wscalar::from)
            .into()
    }

    /// Convert scalar to big-endian byte representation
    pub fn to_bytes(&self) -> [u8; Self::SIZE_BYTES] {
        self.0.to_bytes().into()
    }
    /// Return true if the scalar is 0
    pub fn is_zero(&self) -> bool {
        self.0.is_zero().into()
    }
}

impl From<GroupSizedBytes> for Wscalar {
    fn from(b: GroupSizedBytes) -> Self {
        let sl: &[u8; 32] = &b.0;
        let s = <Scalar as Reduce<FieldBytes>>::reduce(&(*sl).into());
        Wscalar(s)
    }
}

impl From<&Challenge> for Scalar {
    fn from(v: &Challenge) -> Self {
        let v: [u8; SOUNDNESS_BYTES] = *v.0 .0;
        let mut prefix = [0u8; 32];
        prefix[32 - SOUNDNESS_BYTES..].copy_from_slice(&v);
        <Scalar as Reduce<FieldBytes>>::reduce(&prefix.into())
    }
}

impl From<Wscalar> for Base16EncodedBytes {
    fn from(w: Wscalar) -> Self {
        let bytes = w.0.to_bytes();
        let bytes_ref: &[u8] = bytes.as_ref();
        Base16EncodedBytes::new(bytes_ref)
    }
}

impl TryFrom<Base16DecodedBytes> for Wscalar {
    type Error = TryFromSliceError;

    fn try_from(value: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let bytes = value.0;
        GroupSizedBytes::try_from(bytes).map(Into::into)
    }
}

impl core::fmt::Debug for Wscalar {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Wscalar:")?;
        f.write_str(&base16::encode_lower(&(*self.0.to_bytes())))
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {

    use crate::sigma_protocol::GROUP_SIZE;

    use super::Wscalar;
    use core::convert::TryFrom;
    use k256::Scalar;
    use k256::{elliptic_curve::PrimeField, FieldBytes};
    use proptest::{collection::vec, prelude::*};

    impl Arbitrary for Wscalar {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), GROUP_SIZE)
                .prop_filter("must be in group range", |bytes| {
                    let opt: Option<Scalar> =
                        Scalar::from_repr(FieldBytes::try_from(bytes.as_slice()).unwrap()).into();
                    opt.is_some()
                })
                .prop_map(|bytes| {
                    Scalar::from_repr(FieldBytes::try_from(bytes.as_slice()).unwrap())
                        .unwrap()
                        .into()
                })
                .boxed()
        }
    }
}
