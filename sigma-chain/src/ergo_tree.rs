use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

pub struct ErgoTree {}

impl SigmaSerializable for ErgoTree {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: W) -> Result<(), io::Error> {
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        Ok(ErgoTree {})
    }
}
