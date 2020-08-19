use super::{fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};
use k256::Scalar;
#[cfg(test)]
use proptest_derive::Arbitrary;
use std::convert::TryInto;

#[cfg_attr(test, derive(Arbitrary))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Challenge(FiatShamirHash);

impl Into<Scalar> for Challenge {
    fn into(self) -> Scalar {
        let v: [u8; SOUNDNESS_BYTES] = self.0.into();
        // prepend zeroes to 32 bytes (big-endian)
        let mut prefix = vec![0u8; 8];
        prefix.append(&mut v.to_vec());
        Scalar::from_bytes_reduced(prefix.as_slice().try_into().expect("32 bytes"))
    }
}

impl Into<Vec<u8>> for Challenge {
    fn into(self) -> Vec<u8> {
        let arr: [u8; SOUNDNESS_BYTES] = self.0.into();
        arr.to_vec()
    }
}

impl From<FiatShamirHash> for Challenge {
    fn from(fsh: FiatShamirHash) -> Self {
        Challenge(fsh)
    }
}

