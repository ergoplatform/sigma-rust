use super::{fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;
use std::convert::TryFrom;

/// Challenge in Sigma protocol
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg(feature = "json")]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Challenge(pub(crate) FiatShamirHash);

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
        #[allow(clippy::unwrap_used)] // since the size is unchanged
        FiatShamirHash::try_from(res.as_slice()).unwrap().into()
    }

    pub fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(self.0 .0.as_ref())?;
        Ok(())
    }

    pub fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, std::io::Error> {
        let mut chal_bytes: [u8; super::SOUNDNESS_BYTES] = [0; super::SOUNDNESS_BYTES];
        r.read_exact(&mut chal_bytes)?;
        Ok(Challenge::from(FiatShamirHash(Box::new(chal_bytes))))
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
