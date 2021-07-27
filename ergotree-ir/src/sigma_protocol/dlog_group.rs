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

use crate::bigint256::BigInt256;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use k256::elliptic_curve::ff::PrimeField;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{ProjectivePoint, PublicKey, Scalar};
use num_bigint::Sign;
use rand::RngCore;
use std::convert::TryFrom;
use std::ops::{Add, Mul, Neg};

/// Elliptic curve point
#[derive(PartialEq, Clone)]
pub struct EcPoint(ProjectivePoint);

#[allow(clippy::unwrap_used)]
impl std::fmt::Debug for EcPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("EC:")?;
        f.write_str(&base16::encode_lower(
            &self.sigma_serialize_bytes().unwrap(),
        ))
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
/// Use cryptographically secure PRNG (like rand::thread_rng())
pub fn random_scalar_in_group_range(rng: impl RngCore) -> Scalar {
    Scalar::generate_vartime(rng)
}

/// Attempts to create BigInt256 from Scalar
/// Returns None if s > 2^255 - 1
/// Since Scalar is in [0, n) range, where n is the group order
/// (FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141)
/// it might not fit into 256-bit BigInt because BigInt uses 1 bit for sign.
pub fn scalar_to_bigint256(s: Scalar) -> Option<BigInt256> {
    let r_g_array = s.to_bytes();
    let r_b_array: &[u8] = r_g_array.as_slice();
    BigInt256::try_from(r_b_array).ok()
}

/// Attempts to create Scalar from BigInt256
/// Returns None if not in the range [0, modulus).
pub fn bigint256_to_scalar(bi: BigInt256) -> Option<Scalar> {
    let (sign, bytes_be) = bi.to_bytes_be();

    if Sign::Minus == sign {
        return None;
    }

    let bytes = bytes_be.as_slice();
    debug_assert!(bytes.len() <= 32);
    let mut bytes_32 = [0; 32];
    for (i, v) in bytes.iter().enumerate() {
        bytes_32[i] = *v;
    }
    Scalar::from_repr(bytes_32.into())
}

impl SigmaSerializable for EcPoint {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
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
    use rand::thread_rng;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<EcPoint>()) {
            let e: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }

    #[test]
    fn scalar_bigint256_roundtrip() {
        // Shift right to make sure that the MSB is 0, so that the Scalar can be
        // converted to a BigInt256 and back
        let rand_scalar: Scalar = random_scalar_in_group_range(thread_rng()) >> 1;
        let as_bigint256: BigInt256 = scalar_to_bigint256(rand_scalar).unwrap();
        assert_eq!(rand_scalar, bigint256_to_scalar(as_bigint256).unwrap());
    }
}
