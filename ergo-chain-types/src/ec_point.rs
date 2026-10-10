//! Elliptic curve point.

use alloc::string::String;
use core::convert::TryFrom;
use core::ops::{Add, Mul, Neg};
use derive_more::{From, Into};
use elliptic_curve::ops::MulByGenerator;
use k256::elliptic_curve::group::prime::PrimeCurveAffine;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{ProjectivePoint, PublicKey, Scalar};
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use sigma_ser::{ScorexParsingError, ScorexSerializable, ScorexSerializeResult};

/// Elliptic curve point
#[derive(PartialEq, Clone, Copy, Default, From, Into)]
#[cfg_attr(
    feature = "json",
    derive(serde::Serialize, serde::Deserialize),
    serde(into = "String", try_from = "String")
)]
pub struct EcPoint(ProjectivePoint);

#[allow(clippy::unwrap_used)]
impl core::fmt::Debug for EcPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("EC:")?;
        f.write_str(&base16::encode_lower(
            &self.scorex_serialize_bytes().unwrap(),
        ))
    }
}

#[allow(clippy::unwrap_used)]
impl core::fmt::Display for EcPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(&base16::encode_lower(
            &self.scorex_serialize_bytes().unwrap(),
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
            .and_then(|bytes| Self::scorex_parse_bytes(&bytes).ok())
    }
}

impl TryFrom<String> for EcPoint {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        EcPoint::from_base16_str(value)
            .ok_or_else(|| String::from("Ecpoint: error parsing from base16-encoded string"))
    }
}

impl From<EcPoint> for String {
    fn from(value: EcPoint) -> String {
        #[allow(clippy::unwrap_used)]
        {
            let bytes = value.scorex_serialize_bytes().unwrap();
            base16::encode_lower(&bytes)
        }
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
    EcPoint(ProjectivePoint::GENERATOR)
}

/// The identity(infinity) element
pub const fn identity() -> EcPoint {
    EcPoint(ProjectivePoint::IDENTITY)
}

/// Check if point is identity(infinity) element
pub fn is_identity(ge: &EcPoint) -> bool {
    *ge == identity()
}

/// Calculates the inverse of the given group element
pub fn inverse(ec: &EcPoint) -> EcPoint {
    -*ec
}

/// Raises the base GroupElement to the exponent. The result is another GroupElement.
pub fn exponentiate(base: &EcPoint, exponent: &Scalar) -> EcPoint {
    if !is_identity(base) {
        // we treat EC as a multiplicative group, therefore, exponentiate point is multiply.
        EcPoint(base.0 * exponent)
    } else {
        *base
    }
}

/// Raise the generator g to the exponent. This is faster than exponentiate(&generator(), exponent)
pub fn exponentiate_gen(exponent: &Scalar) -> EcPoint {
    ProjectivePoint::mul_by_generator(exponent).into()
}

impl ScorexSerializable for EcPoint {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
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

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let mut buf = [0; EcPoint::GROUP_SIZE];
        r.read_exact(&mut buf[..])?;
        if buf[0] != 0 {
            let pubkey = PublicKey::from_sec1_bytes(&buf[..]).map_err(|e| {
                ScorexParsingError::Misc(format!("failed to parse PK from bytes: {:?}", e))
            })?;
            Ok(EcPoint(pubkey.to_projective()))
        } else {
            // infinity point
            Ok(EcPoint(ProjectivePoint::IDENTITY))
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
mod golden_tests {
    use super::*;

    /// Golden vectors for the serialization format defined by the reference
    /// implementation's `GroupElementSerializer`
    /// (sigmastate-interpreter, `sigma/serialization/GroupElementSerializer.scala`):
    /// a point is encoded in compressed form as 33 bytes — prefix 0x02 if the
    /// affine Y coordinate is even, 0x03 if it is odd, followed by the 32-byte
    /// big-endian X coordinate; the identity (infinity) point is 33 zero bytes.
    ///
    /// The expected byte strings below were computed independently of this
    /// crate (plain secp256k1 arithmetic), starting from the generator's
    /// well-known compressed encoding, so they pin the wire format itself
    /// rather than this implementation's self-consistency.
    fn golden_vectors() -> Vec<(EcPoint, [u8; EcPoint::GROUP_SIZE])> {
        let g = generator();
        // `Mul` on `EcPoint` is point addition (the group is written
        // multiplicatively), so `g * &g` is the point 2*g.
        let g2 = g * &g;
        let g3 = g2 * &g;
        // 12345*g via double-and-add over the group operation.
        let mut g12345 = identity();
        let mut base = g;
        let mut k = 12345u32;
        while k > 0 {
            if k & 1 == 1 {
                g12345 = g12345 * &base;
            }
            base = base * &base;
            k >>= 1;
        }
        let decode = |hex: &str| -> [u8; EcPoint::GROUP_SIZE] {
            base16::decode(hex).unwrap().try_into().unwrap()
        };
        vec![
            (identity(), [0u8; EcPoint::GROUP_SIZE]),
            (
                g,
                decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"),
            ),
            (
                g2,
                decode("02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5"),
            ),
            (
                g3,
                decode("02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9"),
            ),
            // -g: same X as g, odd Y, hence the 0x03 prefix.
            (
                -g,
                decode("0379be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"),
            ),
            // 12345*g also has odd Y (0x03 prefix).
            (
                g12345,
                decode("03f01d6b9018ab421dd410404cb869072065522bf85734008f105cf385a023a80f"),
            ),
        ]
    }

    #[test]
    fn serialize_matches_reference_format() {
        for (point, expected) in golden_vectors() {
            assert_eq!(point.scorex_serialize_bytes().unwrap(), expected.to_vec());
        }
    }

    #[test]
    fn parse_matches_reference_format() {
        for (point, bytes) in golden_vectors() {
            let parsed = EcPoint::scorex_parse_bytes(&bytes).unwrap();
            assert_eq!(parsed, point);
            // Re-serializing a parsed point reproduces the input byte-exactly.
            assert_eq!(parsed.scorex_serialize_bytes().unwrap(), bytes.to_vec());
        }
    }

    #[test]
    fn parse_zero_prefix_is_identity_regardless_of_trailing_bytes() {
        // Both this implementation and the reference `parse` branch on the
        // first byte only: 0x00 means the identity point, whatever follows.
        let mut bytes = [0xabu8; EcPoint::GROUP_SIZE];
        bytes[0] = 0;
        let parsed = EcPoint::scorex_parse_bytes(&bytes).unwrap();
        assert!(is_identity(&parsed));
        assert_eq!(
            parsed.scorex_serialize_bytes().unwrap(),
            vec![0u8; EcPoint::GROUP_SIZE]
        );
    }

    #[test]
    fn parse_rejects_invalid_encodings() {
        let g_bytes = golden_vectors()[1].1;
        // Truncated input (32 of 33 bytes).
        assert!(EcPoint::scorex_parse_bytes(&g_bytes[..32]).is_err());
        // Uncompressed-point tag 0x04 is not a valid 33-byte encoding.
        let mut bad_tag = g_bytes;
        bad_tag[0] = 0x04;
        assert!(EcPoint::scorex_parse_bytes(&bad_tag).is_err());
        // X coordinate equal to the field prime p is out of range.
        let x_at_p =
            base16::decode("02fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f")
                .unwrap();
        assert!(EcPoint::scorex_parse_bytes(&x_at_p).is_err());
        // X = 5 is a valid field element but x^3 + 7 is not a quadratic
        // residue mod p, so there is no such point on the curve.
        let mut x5 = [0u8; EcPoint::GROUP_SIZE];
        x5[0] = 0x02;
        x5[32] = 5;
        assert!(EcPoint::scorex_parse_bytes(&x5).is_err());
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<EcPoint>()) {
            prop_assert_eq![scorex_serialize_roundtrip(&v), v];
        }

    }
}
