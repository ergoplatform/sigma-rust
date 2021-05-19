use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Collection size
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SizeOf {
    /// Collection
    pub input: Box<Expr>,
}

impl SizeOf {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SInt
    }
}

impl HasStaticOpCode for SizeOf {
    const OP_CODE: OpCode = OpCode::SIZE_OF;
}

impl UnaryOp for SizeOf {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for SizeOf {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SColl(_) => Ok(Self {
                input: input.into(),
            }),
            _ => Err(InvalidArgumentError(format!(
                "Expected SizeOf input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::types::stype_param::STypeVar;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for SizeOf {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(SType::STypeVar(STypeVar::t()).into()),
                depth: 1,
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
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<SizeOf>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
