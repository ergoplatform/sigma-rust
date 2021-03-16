use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Extract serialized bytes of a SigmaProp value
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SigmaPropBytes {
    /// SigmaProp value
    pub input: Box<Expr>,
}

impl SigmaPropBytes {
    pub(crate) const OP_CODE: OpCode = OpCode::SIGMA_PROP_BYTES;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SSigmaProp)?;
        Ok(SigmaPropBytes {
            input: input.into(),
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for SigmaPropBytes {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(SigmaPropBytes::new(Expr::sigma_parse(r)?)?)
    }
}

#[cfg(feature = "arbitrary")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::constant::Constant;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::sigma_protocol::sigma_boolean::SigmaProp;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(8))]

        #[test]
        fn ser_roundtrip(v in any::<SigmaProp>()) {
            let input: Constant = v.into();
            let e: Expr = SigmaPropBytes {
                input: Box::new(input.into()),
            }
            .into();
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
