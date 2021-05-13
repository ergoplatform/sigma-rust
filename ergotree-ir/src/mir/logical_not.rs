use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

/// Logical NOT (inverts the input)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct LogicalNot {
    /// Input expr of SBoolean type
    pub input: Box<Expr>,
}

impl LogicalNot {
    pub(crate) const OP_CODE: OpCode = OpCode::LOGICAL_NOT;

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl UnaryOp for LogicalNot {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for LogicalNot {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SBoolean)?;
        Ok(Self {
            input: input.into(),
        })
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for LogicalNot {
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
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<LogicalNot>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
