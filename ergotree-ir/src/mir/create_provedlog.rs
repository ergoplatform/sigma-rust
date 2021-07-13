use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Create ProveDlog from PK
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateProveDlog {
    /// GroupElement (PK)
    pub input: Box<Expr>,
}

impl CreateProveDlog {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SSigmaProp
    }
}

impl HasStaticOpCode for CreateProveDlog {
    const OP_CODE: OpCode = OpCode::PROVE_DLOG;
}

impl UnaryOp for CreateProveDlog {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for CreateProveDlog {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SGroupElement)?;
        Ok(CreateProveDlog {
            input: input.into(),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use sigma_test_util::force_any_val_with;

    use crate::mir::constant::arbitrary::ArbConstantParams;
    use crate::mir::constant::Constant;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = CreateProveDlog::try_build(
            force_any_val_with::<Constant>(ArbConstantParams::Exact(SType::SGroupElement)).into(),
        )
        .unwrap()
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
