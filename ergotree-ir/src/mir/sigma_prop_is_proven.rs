use alloc::boxed::Box;

use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Represents execution of Sigma protocol that validates the given input SigmaProp.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SigmaPropIsProven {
    /// SigmaProp value
    pub input: Box<Expr>,
}

impl SigmaPropIsProven {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }
}

impl HasStaticOpCode for SigmaPropIsProven {
    const OP_CODE: OpCode = OpCode::SIGMA_PROP_IS_PROVEN;
}

impl OneArgOp for SigmaPropIsProven {
    fn input(&self) -> &Expr {
        &self.input
    }
    fn input_mut(&mut self) -> &mut Expr {
        &mut self.input
    }
}

impl OneArgOpTryBuild for SigmaPropIsProven {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SSigmaProp)?;
        Ok(SigmaPropIsProven {
            input: input.into(),
        })
    }
}

#[cfg(feature = "arbitrary")]
#[cfg(test)]
#[allow(clippy::panic)]
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
            let e: Expr = SigmaPropIsProven {
                input: Box::new(input.into()),
            }
            .into();
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
