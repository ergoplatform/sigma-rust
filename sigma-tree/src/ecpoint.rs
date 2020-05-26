use secp256k1::PublicKey;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;

// TODO: unwrap and make type alias?
#[derive(PartialEq, Eq, Debug)]
pub struct EcPoint(pub PublicKey);

impl SigmaSerializable for EcPoint {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.write_all(&self.0.serialize())?;
        Ok(())
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let mut bytes = [0; 33];
        r.read_exact(&mut bytes[..])?;
        let pk = PublicKey::from_slice(&bytes[..])
            .map_err(|_| SerializationError::Misc("invalid secp256k1 compressed public key"))?;
        Ok(EcPoint(pk))
    }
}
