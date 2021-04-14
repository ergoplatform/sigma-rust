use super::{fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;
use k256::Scalar;
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;
use std::convert::TryFrom;
use std::convert::TryInto;

/// Challenge in Sigma protocol
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Challenge(FiatShamirHash);

impl From<Challenge> for Scalar {
    fn from(v: Challenge) -> Self {
        let v: [u8; SOUNDNESS_BYTES] = v.0.into();
        // prepend zeroes to 32 bytes (big-endian)
        let mut prefix = vec![0u8; 8];
        prefix.append(&mut v.to_vec());
        Scalar::from_bytes_reduced(prefix.as_slice().try_into().expect("32 bytes"))
    }
}

impl Challenge {
    pub fn secure_random() -> Self {
        Self(FiatShamirHash::secure_random())
    }

    pub fn xor(self, other: Challenge) -> Self {
        let this: [u8; SOUNDNESS_BYTES] = self.0.into();
        let that: [u8; SOUNDNESS_BYTES] = other.0.into();
        let res: Vec<u8> = this
            .iter()
            .zip(that.iter())
            .map(|(&x1, &x2)| x1 ^ x2)
            .collect();
        FiatShamirHash::try_from(res.as_slice()).unwrap().into()
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

impl SigmaSerializable for Challenge {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(self.0 .0.as_ref())
    }

    fn sigma_parse<R: SigmaByteRead>(_r: &mut R) -> Result<Self, SerializationError> {
        todo!()
    }
}
