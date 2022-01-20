//!  By Leonid Reyzin
//!  This is free and unencumbered software released into the public domain.
//!
//!  Anyone is free to copy, modify, publish, use, compile, sell, or
//!  distribute this software, either in source code form or as a compiled
//!  binary, for any purpose, commercial or non-commercial, and by any
//!  means.
//!
//!  In jurisdictions that recognize copyright laws, the author or authors
//!  of this software dedicate any and all copyright interest in the
//!  software to the public domain. We make this dedication for the benefit
//!  of the public at large and to the detriment of our heirs and
//!  successors. We intend this dedication to be an overt act of
//!  relinquishment in perpetuity of all present and future rights to this
//!  software under copyright law.
//!
//!  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
//!  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//!  MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
//!  IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
//!  OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
//!  ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
//!  OTHER DEALINGS IN THE SOFTWARE.
//!
//!  For more information, please refer to <http://unlicense.org>

use thiserror::Error;

use crate::{gf2_192::Gf2_192, Gf2_192Error};

/// Byte representation of the coefficients of `Gf2_192Poly`. Each coefficient is a `[u8; 24]`
/// representation of a `Gf2_192` instance. Note that the degree zero coefficient is provided
/// separately in the `coeff0` field.
pub struct CoefficientsByteRepr<'a> {
    /// Coefficient of constant term of degree zero.
    pub coeff0: [u8; 24],
    /// Ordered coefficients of the non-zero-degree terms, starting with degree 1.
    pub more_coeffs: &'a [u8],
}

#[derive(PartialEq, Eq, Clone, Debug)]
/// A polynomial with coefficients in GF(2^192), whose domain ranges from `[i8::MIN, i8::MAX]`.
pub struct Gf2_192Poly {
    /// Coefficients of the polynomial. Must be non-empty.
    coefficients: Vec<Gf2_192>,
    /// Upper bound on the degree of the polynomial.
    degree: usize,
}

/// `Gf2_192Poly` error
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum Gf2_192PolyError {
    /// `Gf2_192Poly` interpolation error
    #[error("`Gf2_192Poly::interpolation`: `points.len() != values.len()`")]
    InterpolatePointsAndValuesLengthDiffer,
}

impl Gf2_192Poly {
    /// Create the unique lowest-degree interpolating polynomial that passes through
    /// `(0, value_at_zero)` and `(points[i], values[i])` for all `i = 0, ..(points.len() - 1)`. i.e.
    ///  if the returned polynomial is denoted by `f` then
    ///   - `f(0) == value_at_zero`
    ///   - `f(points[i]) == values[i]` for all `i = 0, ..(points.len() - 1)`
    ///
    /// Assumptions:
    ///  - Elements of `points` must be distinct `u8` values and must not contain `0`.
    ///  - `points.len() == values.len()`. Note that `points` and `values` can be empty, resulting
    ///    in a constant polynomial with value `value_at_zero`.
    pub fn interpolate(
        points: &[u8],
        values: &[Gf2_192],
        value_at_zero: Gf2_192,
    ) -> Result<Gf2_192Poly, Gf2_192PolyError> {
        if points.len() != values.len() {
            return Err(Gf2_192PolyError::InterpolatePointsAndValuesLengthDiffer);
        }

        let result_degree = values.len();

        let mut result = Gf2_192Poly::make_constant(result_degree, 0);
        let mut vanishing_poly = Gf2_192Poly::make_constant(result_degree, 1);

        for i in 0..points.len() {
            let mut t = result.evaluate(points[i]);
            let mut s = vanishing_poly.evaluate(points[i]);

            // need to find r such that currentValue+r*valueOfVanishingPoly = values[i]
            t = t + values[i];
            s = Gf2_192::invert(s);
            t = t * s;

            result.add_monic_times_constant(vanishing_poly.clone(), t);

            // Note: internally the domain of the polynomial is not the set of `u8` values but
            // rather the set of `i8` values. This is because the original implementation from
            // Reyzin was in Java, a language which does not have unsigned integers.
            vanishing_poly.multiply_by_linear_binomial(points[i] as i8);
        }

        // Last point is at 0
        let mut t = result.coefficients[0]; // evaluating at 0 is easy
        let mut s = vanishing_poly.coefficients[0]; // evaluating at 0 is easy

        // need to find r such that currentValue+r*valueOfVanishingPoly = valueAt0]
        t = t + value_at_zero;
        s = Gf2_192::invert(s);
        t = t * s;
        result.add_monic_times_constant(vanishing_poly, t);

        Ok(result)
    }

    /// Evaluates polynomial at the given point `x`.
    pub fn evaluate(&self, x: u8) -> Gf2_192 {
        // Note: internally the domain of the polynomial is not the set of `u8` values but rather
        // the set of `i8` values. This is because the original implementation from Reyzin was in
        // Java, a language which does not have unsigned integers.
        let mut res = self.coefficients[self.degree];
        if self.degree > 0 {
            for d in (0..=(self.degree - 1)).rev() {
                res = Gf2_192::mul_by_i8(res, x as i8);
                res = res + self.coefficients[d];
            }
        }
        res
    }

    /// Returns Vec<u8> consisting of the concatenation of all the coefficients of the polynomial
    /// NOT including the degree-zero coefficient. Each coefficient takes 24 bytes for a total of
    /// `self.degree * 24` bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res: Vec<_> = std::iter::repeat(0).take(self.degree * 24).collect();
        for i in 1..=self.degree {
            #[allow(clippy::unwrap_used)]
            self.coefficients[i]
                .to_i8_slice(&mut res, (i - 1) * 24)
                .unwrap();
        }
        res.into_iter().map(|x| x as u8).collect()
    }

    /// Adds r*p to `self`. Assumes:
    ///  - p is monic
    ///  - self.coefficients.len() > p.degree
    ///  - p.degree == self.degree + 1 or (self == 0 and p == 1)
    fn add_monic_times_constant(&mut self, p: Gf2_192Poly, r: Gf2_192) {
        let mut _t = Gf2_192::new();
        for i in 0..p.degree {
            _t = p.coefficients[i] * r;
            self.coefficients[i] = self.coefficients[i] + _t;
        }
        self.degree = p.degree;
        self.coefficients[self.degree] = r;
    }

    /// Multiply `self` by `x + r`. Assumes that `self` is monic (i.e.
    /// `self.coefficients[self.degree] == 1`)
    fn multiply_by_linear_binomial(&mut self, r: i8) {
        self.degree += 1;
        self.coefficients[self.degree] = Gf2_192::from(1);
        for i in (1..self.degree).rev() {
            self.coefficients[i] = Gf2_192::mul_by_i8(self.coefficients[i], r);
            self.coefficients[i] = self.coefficients[i] + self.coefficients[i - 1];
        }
        self.coefficients[0] = Gf2_192::mul_by_i8(self.coefficients[0], r);
    }

    /// Constructs a constant polynomial (degree 0) which takes value of `constant_term`.
    /// `max_degree` specifies the maximum degree of the polynomial (to allocate space).
    fn make_constant(max_degree: usize, constant_term: i32) -> Gf2_192Poly {
        let mut coefficients: Vec<_> = std::iter::repeat_with(Gf2_192::new)
            .take(max_degree + 1)
            .collect();
        coefficients[0] = Gf2_192::from(constant_term);
        let degree = 0;
        Gf2_192Poly {
            degree,
            coefficients,
        }
    }
}

impl<'a> TryFrom<CoefficientsByteRepr<'a>> for Gf2_192Poly {
    type Error = Gf2_192Error;

    /// Constructs the polynomial given the byte array representation of the coefficients. Note that
    /// the coefficient of degree zero is provided separately (see [`CoefficientsByteRepr`]).
    fn try_from(c: CoefficientsByteRepr<'a>) -> Result<Self, Self::Error> {
        let degree = c.more_coeffs.len() / 24;
        let mut coefficients = Vec::with_capacity(degree + 1);
        coefficients.push(Gf2_192::from(c.coeff0));

        for i in 1..=degree {
            coefficients.push(Gf2_192::try_from(&c.more_coeffs[(i - 1) * 24..])?);
        }
        Ok(Gf2_192Poly {
            degree,
            coefficients,
        })
    }
}

/// The following tests closely match those in `ScoreXFoundation/sigmastate-interpreter`.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_interpolation() {
        // Try with arrays of length 0

        // Constant polynomial with value 0
        let mut p = Gf2_192Poly::interpolate(&[], &[], Gf2_192::new()).unwrap();
        assert!(p.evaluate(0).is_zero());
        assert!(p.evaluate(5).is_zero());

        // Constant polynomial with value 17
        let val_17 = Gf2_192::from(17);
        p = Gf2_192Poly::interpolate(&[], &[], val_17).unwrap();
        assert_eq!(p.evaluate(0), val_17);
        assert_eq!(p.evaluate(5), val_17);

        let mut rng = thread_rng();

        for len in 1..100 {
            let mut points: Vec<_> = std::iter::repeat(0).take(len).collect();
            // Generate a byte that's not an element of `points` nor 0
            let mut j = 0;
            while j < points.len() {
                let b: u8 = rng.gen();
                if b != 0 && !points.contains(&b) {
                    points[j] = b;
                    j += 1;
                }
            }
            // Generate random elements for `values`
            let mut values: Vec<_> = Vec::with_capacity(len);
            for _ in 0..len {
                let b: [i8; 24] = rng.gen();
                values.push(Gf2_192::from(b));
            }

            let b: [i8; 24] = rng.gen();
            let value_at_zero = Gf2_192::from(b);
            let res = Gf2_192Poly::interpolate(&points, &values, value_at_zero).unwrap();

            // Check that interpolating function hits `values[i]` for every `points[i]`.
            for i in 0..points.len() {
                assert_eq!(res.evaluate(points[i]), values[i]);
            }

            // Check the interpolating function at zero
            assert_eq!(res.evaluate(0), value_at_zero);
        }
    }
}
