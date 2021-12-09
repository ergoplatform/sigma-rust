use super::{challenge::Challenge, fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct GF2_192(gf2_192::gf2_192::GF2_192);

impl From<GF2_192> for Challenge {
    fn from(e: GF2_192) -> Self {
        let bytes = <[u8; 24]>::from(e.0);
        Challenge(FiatShamirHash(Box::new(bytes)))
    }
}

impl From<Challenge> for GF2_192 {
    fn from(c: Challenge) -> Self {
        let bytes: [u8; SOUNDNESS_BYTES] = c.0.into();
        GF2_192(gf2_192::gf2_192::GF2_192::from(bytes))
    }
}
