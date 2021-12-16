use std::convert::TryFrom;

use super::{challenge::Challenge, fiat_shamir::FiatShamirHash, SOUNDNESS_BYTES};
use gf2_192::{
    gf2_192::Gf2_192,
    gf2_192poly::{CoefficientsByteRepr, Gf2_192Poly},
};

impl From<Gf2_192> for Challenge {
    fn from(e: Gf2_192) -> Self {
        let bytes = <[u8; 24]>::from(e);
        Challenge(FiatShamirHash(Box::new(bytes)))
    }
}

impl From<Challenge> for Gf2_192 {
    fn from(c: Challenge) -> Self {
        let bytes: [u8; SOUNDNESS_BYTES] = c.0.into();
        Gf2_192::from(bytes)
    }
}

/// Create a `Gf2_192Poly` instance from given challenge and byte array of other coefficients.
/// Note:
///  - `challenge` is viewed as an element of `GF(2^192)` (see the trait impls above) and serves as
///    the degree-zero coefficient in the resulting polynomial.
///  - `other_coefficients` represents the other coefficients of the polynomial starting from degree
///    one. Each coefficient is represented by slice of `[u8; 24]`.
pub(crate) fn gf2_192poly_from_byte_array(
    challenge: Challenge,
    other_coefficients: Vec<u8>,
) -> Result<Gf2_192Poly, gf2_192::Gf2_192Error> {
    let c0 = Gf2_192::from(challenge);
    let coeff0 = <[u8; 24]>::from(c0);
    let cc = CoefficientsByteRepr {
        coeff0,
        more_coeffs: &other_coefficients, 
    };
    Gf2_192Poly::try_from(cc)
}
