//! Wrapper for Scalar
//! mainly for Arbitrary impl and JSON encoding

use std::array::TryFromSliceError;
use std::convert::TryFrom;

use derive_more::From;
use derive_more::Into;
use elliptic_curve::generic_array::GenericArray;
use elliptic_curve::ops::Reduce;
use ergo_chain_types::Base16DecodedBytes;
use ergo_chain_types::Base16EncodedBytes;
use k256::Scalar;
use k256::U256;

use super::challenge::Challenge;
use super::GroupSizedBytes;
use super::SOUNDNESS_BYTES;

#[derive(PartialEq, Eq, Debug, From, Into, Clone)]
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
    /// Returns a reference to underlying Scalar
    pub fn as_scalar_ref(&self) -> &Scalar {
        &self.0
    }
}

impl From<GroupSizedBytes> for Wscalar {
    fn from(b: GroupSizedBytes) -> Self {
        let sl: &[u8] = b.0.as_ref();
        let s = <Scalar as Reduce<U256>>::from_be_bytes_reduced(GenericArray::clone_from_slice(sl));
        Wscalar(s)
    }
}

impl From<Challenge> for Scalar {
    fn from(v: Challenge) -> Self {
        let v: [u8; SOUNDNESS_BYTES] = v.0.into();
        // prepend zeroes to 32 bytes (big-endian)
        let mut prefix = vec![0u8; 8];
        prefix.append(&mut v.to_vec());
        <Scalar as Reduce<U256>>::from_be_bytes_reduced(GenericArray::clone_from_slice(&prefix))
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

#[cfg(feature = "arbitrary")]
mod arbitrary {

    use crate::sigma_protocol::GROUP_SIZE;

    use super::Wscalar;
    use elliptic_curve::{generic_array::GenericArray, PrimeField};
    use k256::Scalar;
    use proptest::{collection::vec, prelude::*};

    impl Arbitrary for Wscalar {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), GROUP_SIZE)
                .prop_filter("must be in group range", |bytes| {
                    let opt: Option<Scalar> =
                        Scalar::from_repr(GenericArray::clone_from_slice(bytes)).into();
                    opt.is_some()
                })
                .prop_map(|bytes| {
                    Scalar::from_repr(GenericArray::clone_from_slice(&bytes))
                        .unwrap()
                        .into()
                })
                .boxed()
        }
    }
}
