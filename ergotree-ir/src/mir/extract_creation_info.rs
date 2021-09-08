use crate::serialization::op_code::OpCode;
use crate::types::stuple::STuple;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Tuple of height when block got included into the blockchain and transaction identifier with
/// box index in the transaction outputs serialized to the byte array.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractCreationInfo {
    /// Box (SBox type)
    pub input: Box<Expr>,
}

impl ExtractCreationInfo {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::STuple(STuple::pair(SType::SInt, SType::SColl(SType::SByte.into())))
    }
}

impl HasStaticOpCode for ExtractCreationInfo {
    const OP_CODE: OpCode = OpCode::EXTRACT_CREATION_INFO;
}

impl OneArgOp for ExtractCreationInfo {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for ExtractCreationInfo {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SBox)?;
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
