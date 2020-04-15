use crate::box_id::BoxId;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
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

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {

        #[test]
        fn data_input_roundtrip(i in any::<DataInput>()) {
            let mut data = Vec::new();
            i.sigma_serialize(&mut data)?;
            let decoded = DataInput::sigma_parse(&data[..])?;
            prop_assert_eq![i, decoded];
        }
    }
}
