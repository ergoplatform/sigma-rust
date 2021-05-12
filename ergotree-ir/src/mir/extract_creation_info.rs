use crate::serialization::op_code::OpCode;
use crate::types::stuple::STuple;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;

/// Tuple of height when block got included into the blockchain and transaction identifier with
/// box index in the transaction outputs serialized to the byte array.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractCreationInfo {
    /// Box (SBox type)
    pub input: Box<Expr>,
}

impl ExtractCreationInfo {
    pub(crate) const OP_CODE: OpCode = OpCode::EXTRACT_CREATION_INFO;

    /// Type
    pub fn tpe(&self) -> SType {
        SType::STuple(STuple::pair(SType::SInt, SType::SColl(SType::SByte.into())))
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl UnaryOp for ExtractCreationInfo {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for ExtractCreationInfo {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError>
    where
        Self: Sized,
    {
        input.check_post_eval_tpe(SType::SBox)?;
        Ok(ExtractCreationInfo {
            input: input.into(),
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
