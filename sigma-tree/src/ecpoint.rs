use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;

// TODO: implement
#[derive(PartialEq, Eq, Debug)]
pub struct EcPointType {}

impl SigmaSerializable for EcPointType {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: &mut W) -> Result<(), io::Error> {
        Ok(())
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: &mut R) -> Result<Self, SerializationError> {
        Ok(EcPointType {})
    }
}
