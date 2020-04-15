use crate::ergo_box::BoxId;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

pub struct DataInput {
    pub box_id: BoxId,
}

impl SigmaSerializable for DataInput {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        self.box_id.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        let box_id = BoxId::sigma_parse(r)?;
        Ok(DataInput { box_id })
    }
}
