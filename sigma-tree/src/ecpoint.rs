use k256::{
    arithmetic::{AffinePoint, ProjectivePoint, Scalar},
    PublicKey,
};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::convert::TryInto;
use std::io;

#[derive(PartialEq, Debug)]
pub struct EcPoint(ProjectivePoint);

impl EcPoint {
    pub const GROUP_SIZE: usize = 33;

    pub fn random() -> EcPoint {
        let scalar = loop {
            // Generate a new secret key using the operating system's
            // cryptographically secure random number generator
            let sk = k256::SecretKey::generate();
            let bytes: [u8; 32] = sk
                .secret_scalar()
                .as_ref()
                .as_slice()
                .try_into()
                .expect("expected 32 bytes");
            // Returns None if the byte array does not contain
            // a big-endian integer in the range [0, n), where n is group order.
            let maybe_scalar = Scalar::from_bytes(bytes);
            if bool::from(maybe_scalar.is_some()) {
                break maybe_scalar.unwrap();
            }
        };
        // we treat EC as a multiplicative group, therefore, exponentiate point is multiply.
        let pkp = ProjectivePoint::generator() * &scalar;
        EcPoint(pkp)
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
