use secp256k1::PublicKey;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;

#[derive(PartialEq, Eq, Debug)]
pub struct EcPoint(pub PublicKey);

impl EcPoint {
    pub const PUBLIC_KEY_SIZE: usize = secp256k1::constants::PUBLIC_KEY_SIZE;
}

impl SigmaSerializable for EcPoint {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_all(&self.0.serialize())?;
        Ok(())
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let mut bytes = [0; EcPoint::PUBLIC_KEY_SIZE];
        r.read_exact(&mut bytes[..])?;
        let pk = PublicKey::from_slice(&bytes[..])
            .map_err(|_| SerializationError::Misc("invalid secp256k1 compressed public key"))?;
        Ok(EcPoint(pk))
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate secp256k1;

    use super::*;
    use proptest::prelude::*;
    use rand::thread_rng;
    use secp256k1::Secp256k1;
    use sigma_ser::test_helpers::*;

    impl Arbitrary for EcPoint {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<i32>())
                .prop_map(|_| {
                    let secp = Secp256k1::new();
                    let (_, public_key) = secp.generate_keypair(&mut thread_rng());
                    EcPoint(public_key)
                })
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
