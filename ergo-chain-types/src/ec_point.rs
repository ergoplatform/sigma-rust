//! Elliptic curve point.

use derive_more::From;
use elliptic_curve::group::prime::PrimeCurveAffine;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{ProjectivePoint, PublicKey, Scalar};
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use sigma_ser::{ScorexParsingError, ScorexSerializable, ScorexSerializeResult};
use std::convert::TryFrom;
use std::ops::{Add, Mul, Neg};

/// Elliptic curve point
#[derive(PartialEq, Clone, Default, From)]
#[cfg_attr(
    feature = "json",
    derive(serde::Serialize, serde::Deserialize),
    serde(into = "String", try_from = "String")
)]
pub struct EcPoint(ProjectivePoint);

#[allow(clippy::unwrap_used)]
impl std::fmt::Debug for EcPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("EC:")?;
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
