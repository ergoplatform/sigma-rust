use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stuple::STuple;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Tuple of height when block got included into the blockchain and transaction identifier with
/// box index in the transaction outputs serialized to the byte array.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractCreationInfo {
    /// Box (SBox type)
    pub input: Box<Expr>,
}

impl ExtractCreationInfo {
    pub(crate) const OP_CODE: OpCode = OpCode::EXTRACT_CREATION_INFO;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SBox)?;
        Ok(ExtractCreationInfo {
            input: input.into(),
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::STuple(STuple::pair(SType::SInt, SType::SColl(SType::SByte.into())))
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for ExtractCreationInfo {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(ExtractCreationInfo {
            input: Expr::sigma_parse(r)?.into(),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractCreationInfo {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
