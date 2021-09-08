use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Extract serialized bytes of a SigmaProp value
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SigmaPropBytes {
    /// SigmaProp value
    pub input: Box<Expr>,
}

impl SigmaPropBytes {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }
}

impl HasStaticOpCode for SigmaPropBytes {
    const OP_CODE: OpCode = OpCode::SIGMA_PROP_BYTES;
}

impl OneArgOp for SigmaPropBytes {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for SigmaPropBytes {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SSigmaProp)?;
        Ok(SigmaPropBytes {
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
            let e: Expr = SigmaPropBytes {
                input: Box::new(input.into()),
            }
            .into();
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
