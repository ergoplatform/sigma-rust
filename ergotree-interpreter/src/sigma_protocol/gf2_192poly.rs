// TODO: remove after all todo! are implemented
#![allow(clippy::todo)]

use std::convert::TryFrom;

use gf2_192::gf2_192poly::CoefficientsByteRepr;

use super::challenge::Challenge;
use super::gf2_192::Gf2_192;

/// A polynomial with coefficients in GF(2^192)
#[derive(PartialEq, Debug, Clone)]
pub struct Gf2_192Poly(gf2_192::gf2_192poly::Gf2_192Poly);

impl Gf2_192Poly {
    pub(crate) fn from_byte_array(
        challenge: Challenge,
        other_coefficients: Vec<u8>,
    ) -> Result<Self, gf2_192::Gf2_192Error> {
        let c0 = Gf2_192::from(challenge).0;
        let coeff0 = <[i8; 24]>::from(c0);
        let more_coeffs: Vec<_> = other_coefficients.into_iter().map(|x| x as i8).collect();
        let cc = CoefficientsByteRepr {
            coeff0,
            more_coeffs: &more_coeffs,
        };
        Ok(Gf2_192Poly(gf2_192::gf2_192poly::Gf2_192Poly::try_from(
            cc,
        )?))
    }

    /// Evaluates polynomial at the given point `idx` (Note that the domain of the polynomial is
    /// actually the set of values spanned by the `i8` type. We simply map `idx: u8` to this set via
    /// a type cast)
    pub(crate) fn evaluate(&self, idx: u8) -> Gf2_192 {
        Gf2_192(self.0.evaluate(idx as i8))
    }

    /// Create the unique lowest-degree interpolating polynomial that passes through
    /// `(0, value_at_zero)` and `(points[i], values[i])` for all `i = 0, ..(points.len() - 1)`.
    /// Assumptions:
    ///  - Elements of `points` must be distinct and must not contain `0`.
    ///  - `points.len() == values.len()`. Note that `points` and `values` can be empty, resulting
    ///    in a constant polynomial with value `value_at_zero`.
    pub(crate) fn interpolate(
        points: Vec<u8>,
        values: Vec<Gf2_192>,
        value_at_zero: Gf2_192,
    ) -> Result<Self, gf2_192::Gf2_192Error> {
        let points: Vec<_> = points.into_iter().map(|x| x as i8).collect();
        let values: Vec<_> = values.into_iter().map(|e| e.0).collect();
        gf2_192::gf2_192poly::Gf2_192Poly::interpolate(&points, &values, value_at_zero.0)
            .map(Gf2_192Poly)
            .map_err(gf2_192::Gf2_192Error::Gf2_192PolyError)
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        self.0.to_i8_vec().into_iter().map(|b| b as u8).collect()
    }
}
