use crate::data_input::DataInput;
use crate::ergo_box::ErgoBoxCandidate;
use crate::input::Input;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::convert::TryFrom;
use std::io;

pub struct Transaction {
    pub inputs: Vec<Input>,
    pub data_inputs: Vec<DataInput>,
    pub outputs: Vec<ErgoBoxCandidate>,
}

impl SigmaSerializable for Transaction {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        w.put_u16(u16::try_from(self.inputs.len()).unwrap())?;
        self.inputs
            .iter()
            .try_for_each(|i| i.sigma_serialize(&mut w))?;
        // TODO: continue with the rest vc
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        unimplemented!()
    }
}
