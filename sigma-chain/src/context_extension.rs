use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::io;

pub struct ContextExtension {
    pub values: HashMap<u8, Vec<u8>>,
}

impl SigmaSerializable for ContextExtension {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: W) -> Result<(), io::Error> {
        assert!(self.values.is_empty(), "implemented only for empty");
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        Ok(ContextExtension {
            values: HashMap::new(),
        })
    }
}
