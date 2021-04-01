//! FIXME

use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
//use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
/// FIXME: DOC
pub struct SubstConstants {
    /// FIXME: DOC
    pub script_bytes: Box<Expr>,
    /// FIXME: DOC
    pub positions: Box<Expr>,
    /// FIXME: DOC
    pub new_values: Box<Expr>,
}

impl SubstConstants {
    pub(crate) const OP_CODE: OpCode = OpCode::SUBST_CONSTANTS;

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for SubstConstants {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.script_bytes.sigma_serialize(w)?;
        self.positions.sigma_serialize(w)?;
        self.new_values.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let script_bytes = Expr::sigma_parse(r)?.into();
        let positions = Expr::sigma_parse(r)?.into();
        let new_values = Expr::sigma_parse(r)?.into();
        Ok(Self {
            script_bytes,
            positions,
            new_values,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    impl Arbitrary for SubstConstants {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Box<Expr>>(), any::<Box<Expr>>(), any::<Box<Expr>>())
                .prop_map(|(script_bytes, positions, new_values)| Self {
                    script_bytes,
                    positions,
                    new_values,
                })
                .boxed()
        }
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<SubstConstants>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
