use crate::context_extension::ContextExtension;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

pub struct ProverResult {
    pub proof: Vec<u8>,
    pub extension: ContextExtension,
}

impl SigmaSerializable for ProverResult {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        w.put_u16(self.proof.len() as u16)?;
        w.write_all(&self.proof)?;
        self.extension.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let proof_len = r.get_u16()?;
        let mut proof = vec![0; proof_len as usize];
        r.read_exact(&mut proof)?;
        let extension = ContextExtension::sigma_parse(r)?;
        Ok(ProverResult { proof, extension })
    }
}
