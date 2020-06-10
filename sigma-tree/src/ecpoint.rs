use crate::sigma_protocol::DlogProverInput;
use k256::{
    arithmetic::{AffinePoint, ProjectivePoint, Scalar},
    PublicKey,
};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;

#[derive(PartialEq, Debug, Clone)]
pub struct EcPoint(ProjectivePoint);

impl EcPoint {
    pub const GROUP_SIZE: usize = 33;

    pub fn random() -> EcPoint {
        let sk = DlogProverInput::random();
        EcPoint::generator().exponentiate(&sk.w)
    }

    pub fn generator() -> EcPoint {
        EcPoint(ProjectivePoint::generator())
    }

    pub fn is_infinity(&self) -> bool {
        let identity = ProjectivePoint::identity();
        self.0 == identity
    }

    pub fn exponentiate(&self, exponent: &Scalar) -> EcPoint {
        if !self.is_infinity() {
            // TODO: check if exponent is negative
            // see reference impl https://github.com/ScorexFoundation/sigmastate-interpreter/blob/ec71a6f988f7412bc36199f46e7ad8db643478c7/sigmastate/src/main/scala/sigmastate/basics/BcDlogGroup.scala#L201

            // we treat EC as a multiplicative group, therefore, exponentiate point is multiply.
            EcPoint(self.0 * exponent)
        } else {
            self.clone()
        }
    }
}

impl Eq for EcPoint {}

impl SigmaSerializable for EcPoint {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        let caff = self.0.to_affine();
        if bool::from(caff.is_some()) {
            let pubkey = caff.unwrap().to_compressed_pubkey();
            w.write_all(pubkey.as_bytes())?;
        } else {
            // infinity point
            let zeroes = [0u8; EcPoint::GROUP_SIZE];
            w.write_all(&zeroes)?;
        }
        Ok(())
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let mut buf = [0; EcPoint::GROUP_SIZE];
        r.read_exact(&mut buf[..])?;
        if buf[0] != 0 {
            let pubkey = PublicKey::from_bytes(&buf[..])
                .ok_or(SerializationError::Misc("failed to parse PK from bytes"))?;
            let cp = AffinePoint::from_pubkey(&pubkey);
            if bool::from(cp.is_none()) {
                Err(SerializationError::Misc(
                    "failed to get affine point from PK",
                ))
            } else {
                Ok(EcPoint(ProjectivePoint::from(cp.unwrap())))
            }
        } else {
            // infinity point
            Ok(EcPoint(ProjectivePoint::identity()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::test_helpers::*;

    impl Arbitrary for EcPoint {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                prop::num::u8::ANY.prop_map(|_| EcPoint(ProjectivePoint::generator())),
                prop::num::u8::ANY.prop_map(|_| EcPoint(ProjectivePoint::identity())),
                prop::num::u8::ANY.prop_map(|_| EcPoint::random()),
            ]
            .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<EcPoint>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
