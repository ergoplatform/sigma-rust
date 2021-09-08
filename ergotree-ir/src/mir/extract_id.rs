use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Box id, Blake2b256 hash of this box's content, basically equals to `blake2b256(bytes)`
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractId {
    /// Box (SBox type)
    pub input: Box<Expr>,
}

impl ExtractId {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }
}

impl HasStaticOpCode for ExtractId {
    const OP_CODE: OpCode = OpCode::EXTRACT_ID;
}

impl OneArgOp for ExtractId {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for ExtractId {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SBox)?;
        Ok(ExtractId {
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
        let e: Expr = ExtractId {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
