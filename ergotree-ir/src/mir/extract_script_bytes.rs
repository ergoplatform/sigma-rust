use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Serialized box guarding script
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractScriptBytes {
    /// Box, type of SBox
    pub input: Box<Expr>,
}

impl ExtractScriptBytes {
    pub(crate) const OP_CODE: OpCode = OpCode::EXTRACT_SCRIPT_BYTES;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SBox)?;
        Ok(ExtractScriptBytes {
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

impl SigmaSerializable for ExtractScriptBytes {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(ExtractScriptBytes::new(Expr::sigma_parse(r)?)?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractScriptBytes {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
