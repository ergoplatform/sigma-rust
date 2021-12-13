use super::{challenge::Challenge, fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Gf2_192(pub(crate) gf2_192::gf2_192::Gf2_192);

impl From<Gf2_192> for Challenge {
    fn from(e: Gf2_192) -> Self {
        let bytes = <[u8; 24]>::from(e.0);
        Challenge(FiatShamirHash(Box::new(bytes)))
    }
}

impl From<Challenge> for Gf2_192 {
    fn from(c: Challenge) -> Self {
        let bytes: [u8; SOUNDNESS_BYTES] = c.0.into();
        Gf2_192(gf2_192::gf2_192::Gf2_192::from(bytes))
    }
}
