use super::{data::DataSerializer, sigma_byte_writer::SigmaByteWrite};
use crate::mir::constant::Constant;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use crate::types::stype::SType;

use std::io;

impl SigmaSerializable for Constant {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.tpe.sigma_serialize(w)?;
        DataSerializer::sigma_serialize(&self.v, w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let tpe = SType::sigma_parse(r)?;
        let v = DataSerializer::sigma_parse(&tpe, r)?;
        Ok(Constant { tpe, v })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::constant::arbitrary::ArbConstantParams;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any_with::<Constant>(ArbConstantParams::AnyWithDepth(4))) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
