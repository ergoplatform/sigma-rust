use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Create ProveDlog from PK
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateProveDlog {
    /// GroupElement (PK)
    pub value: Box<Expr>,
}

impl CreateProveDlog {
    pub(crate) const OP_CODE: OpCode = OpCode::PROVE_DLOG;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(value: Expr) -> Result<Self, InvalidArgumentError> {
        value.check_post_eval_tpe(SType::SGroupElement)?;
        Ok(CreateProveDlog {
            value: value.into(),
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SSigmaProp
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for CreateProveDlog {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.value.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(CreateProveDlog {
            value: Expr::sigma_parse(r)?.into(),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use sigma_test_util::force_any_val_with;

    use crate::mir::constant::arbitrary::ArbConstantParams;
    use crate::mir::constant::Constant;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = CreateProveDlog::new(
            force_any_val_with::<Constant>(ArbConstantParams::Exact(SType::SGroupElement)).into(),
        )
        .unwrap()
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
