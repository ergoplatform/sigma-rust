use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;

/// Serialized box guarding script
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractScriptBytes {
    /// Box, type of SBox
    pub input: Box<Expr>,
}

impl ExtractScriptBytes {
    pub(crate) const OP_CODE: OpCode = OpCode::EXTRACT_SCRIPT_BYTES;

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl UnaryOp for ExtractScriptBytes {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for ExtractScriptBytes {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError>
    where
        Self: Sized,
    {
        input.check_post_eval_tpe(SType::SBox)?;
        Ok(ExtractScriptBytes {
            input: input.into(),
        })
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
