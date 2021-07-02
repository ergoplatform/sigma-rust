//! Embedding of Boolean values to SigmaProp
use crate::has_opcode::HasStaticOpCode;
use crate::mir::unary_op::UnaryOp;
use crate::mir::unary_op::UnaryOpTryBuild;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/** Embedding of Boolean values to SigmaProp values. As an example, this operation allows boolean experesions
 * to be used as arguments of `atLeast(..., sigmaProp(boolExpr), ...)` operation.
 * During execution results to either `TrueProp` or `FalseProp` values of SigmaProp type.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoolToSigmaProp {
    /// Expr of type SBoolean
    pub input: Box<Expr>,
}

impl BoolToSigmaProp {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SSigmaProp
    }
}

impl HasStaticOpCode for BoolToSigmaProp {
    const OP_CODE: OpCode = OpCode::BOOL_TO_SIGMA_PROP;
}

impl UnaryOp for BoolToSigmaProp {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for BoolToSigmaProp {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SBoolean)?;
        Ok(Self {
            input: input.into(),
        })
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for BoolToSigmaProp {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SBoolean,
                depth: args,
            })
            .prop_map(|input| Self {
                input: input.into(),
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {

    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<BoolToSigmaProp>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
