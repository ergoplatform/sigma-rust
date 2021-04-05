use super::{fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};
use k256::Scalar;
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;
use std::convert::TryInto;

/// Challenge in Sigma protocol
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct Challenge(FiatShamirHash);

impl From<Challenge> for Scalar {
    fn from(v: Challenge) -> Self {
        let v: [u8; SOUNDNESS_BYTES] = v.0.into();
        // prepend zeroes to 32 bytes (big-endian)
        let mut prefix = vec![0u8; 8];
        prefix.append(&mut v.to_vec());
        Scalar::from_bytes_reduced(prefix.as_slice().try_into().expect("32 bytes"))
    }
}

impl From<Challenge> for Vec<u8> {
    fn from(v: Challenge) -> Self {
        let arr: [u8; SOUNDNESS_BYTES] = v.0.into();
        arr.to_vec()
    }
}

impl From<FiatShamirHash> for Challenge {
    fn from(fsh: FiatShamirHash) -> Self {
        Challenge(fsh)
    }
}
