//! This is the general interface for the discrete logarithm prime-order group.
//!
//! The discrete logarithm problem is as follows: given a generator g of a finite
//! group G and a random element h in G, find the (unique) integer x such that
//! `g^x = h`.
//!
//! In cryptography, we are interested in groups for which the discrete logarithm problem
//! (Dlog for short) is assumed to be hard. The most known groups of that kind are some Elliptic curve groups.
//!
//! Another issue pertaining elliptic curves is the need to find a suitable mapping that will convert an arbitrary
//! message (that is some binary string) to an element of the group and vice-versa.
//!
//! Only a subset of the messages can be effectively mapped to a group element in such a way that there is a one-to-one
//! injection that converts the string to a group element and vice-versa.
//!
//! On the other hand, any group element can be mapped to some string.

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use k256::elliptic_curve::ff::PrimeField;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{ProjectivePoint, PublicKey, Scalar};
use num_bigint::{BigInt, Sign};
use sigma_ser::vlq_encode;

use std::{
    io,
    ops::{Add, Mul, Neg},
};

/// Elliptic curve point
#[derive(PartialEq, Clone)]
pub struct EcPoint(ProjectivePoint);

impl std::fmt::Debug for EcPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("EC:")?;
        f.write_str(&base16::encode_lower(&self.sigma_serialize_bytes()))
    }
}

impl EcPoint {
    /// Number of bytes to represent any group element as byte array
    pub const GROUP_SIZE: usize = 33;

    /// Attempts to parse from Base16-encoded string
    pub fn from_base16_str(str: String) -> Option<Self> {
        base16::decode(&str)
            .ok()
            .map(|bytes| Self::sigma_parse_bytes(&bytes).ok())
            .flatten()
    }
}

impl Eq for EcPoint {}

impl Mul<&EcPoint> for EcPoint {
    type Output = EcPoint;

    fn mul(self, other: &EcPoint) -> EcPoint {
        EcPoint(ProjectivePoint::add(self.0, &other.0))
    }
}

impl Neg for EcPoint {
    type Output = EcPoint;

    fn neg(self) -> EcPoint {
        EcPoint(ProjectivePoint::neg(self.0))
    }
}

/// The generator g of the group is an element of the group such that, when written multiplicatively, every element
/// of the group is a power of g.
pub fn generator() -> EcPoint {
    EcPoint(ProjectivePoint::generator())
}

/// The identity(infinity) element
pub const fn identity() -> EcPoint {
    EcPoint(ProjectivePoint::identity())
}

/// Check if point is identity(infinity) element
pub fn is_identity(ge: &EcPoint) -> bool {
    *ge == identity()
}

/// Calculates the inverse of the given group element
pub fn inverse(ec: &EcPoint) -> EcPoint {
    -ec.clone()
}

/// Raises the base GroupElement to the exponent. The result is another GroupElement.
pub fn exponentiate(base: &EcPoint, exponent: &Scalar) -> EcPoint {
    if !is_identity(base) {
        // we treat EC as a multiplicative group, therefore, exponentiate point is multiply.
        EcPoint(base.0 * exponent)
    } else {
        base.clone()
    }
}

// /// Creates a random member of this Dlog group
// pub fn random_element() -> EcPoint {
//     let sk = DlogProverInput::random();
//     exponentiate(&generator(), &sk.w)
// }

/// Creates a random scalar, a big-endian integer in the range [0, n), where n is group order
pub fn random_scalar_in_group_range() -> Scalar {
    use rand::rngs::OsRng;
    Scalar::generate_vartime(&mut OsRng)
}

/// Attempts to create BigInt from Scalar
pub fn scalar_to_bigint(s: Scalar) -> BigInt {
    let r_g_array = s.to_bytes();
    let r_b_array: &[u8] = r_g_array.as_slice();
    BigInt::from_bytes_be(Sign::Plus, r_b_array)
}

/// Attempts to create Scalar from BigInt
/// Returns None if not in the range [0, modulus).
pub fn bigint_to_scalar(bi: BigInt) -> Option<Scalar> {
    if num_bigint::Sign::Minus == bi.sign() {
        return None;
    }

    match BigInt::to_biguint(&bi) {
        Some(bui) => {
            let bytes_be = bui.to_bytes_be();
            let bytes = bytes_be.as_slice();

            if bytes.len() > 32 {
                return None;
            }

            let mut bytes_32 = [0; 32];
            for (i, v) in bytes.iter().enumerate() {
                bytes_32[i] = *v;
            }
            Scalar::from_repr(bytes_32.into())
        }
        _ => None,
    }
}

impl SigmaSerializable for EcPoint {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        let caff = self.0.to_affine();
        if caff.is_identity().into() {
            // infinity point
            let zeroes = [0u8; EcPoint::GROUP_SIZE];
            w.write_all(&zeroes)?;
        } else {
            w.write_all(caff.to_encoded_point(true).as_bytes())?;
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let mut buf = [0; EcPoint::GROUP_SIZE];
        r.read_exact(&mut buf[..])?;
        if buf[0] != 0 {
            let pubkey = PublicKey::from_sec1_bytes(&buf[..]).map_err(|e| {
                SigmaParsingError::Misc(format!("failed to parse PK from bytes: {:?}", e))
            })?;
            Ok(EcPoint(pubkey.to_projective()))
        } else {
            // infinity point
            Ok(EcPoint(ProjectivePoint::identity()))
        }
    }
}

/// Arbitrary impl for EcPoint
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for EcPoint {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(generator()),
                Just(identity()), /*Just(random_element()),*/
            ]
            .boxed()
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<EcPoint>()) {
            let e: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }

    #[test]
    fn scalar_bigint_roundtrip() {
        let rand_scalar: Scalar = random_scalar_in_group_range();
        let as_bigint: BigInt = scalar_to_bigint(rand_scalar);
        assert_eq!(rand_scalar, bigint_to_scalar(as_bigint).unwrap());
    }
}
