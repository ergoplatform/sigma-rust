use super::{data::DataSerializer, sigma_byte_writer::SigmaByteWrite};
use crate::mir::constant::Constant;
use crate::serialization::types::TypeCode;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::types::stype::SType;

impl Constant {
    /// Parse constant from byte stream. This function should be used instead of
    /// `sigma_parse` when type code is already read for look-ahead
    pub fn parse_with_type_code<R: SigmaByteRead>(
        r: &mut R,
        t_code: TypeCode,
    ) -> Result<Self, SigmaParsingError> {
        let tpe = SType::parse_with_type_code(r, t_code)?;
        let v = DataSerializer::sigma_parse(&tpe, r)?;
        Ok(Constant { tpe, v })
    }
}
impl SigmaSerializable for Constant {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.tpe.sigma_serialize(w)?;
        DataSerializer::sigma_serialize(&self.v, w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let t_code = TypeCode::sigma_parse(r)?;
        Self::parse_with_type_code(r, t_code)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
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
