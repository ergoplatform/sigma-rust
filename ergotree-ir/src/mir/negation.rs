use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

/// Negation operation on numeric type.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Negation {
    /// Input expr of numeric type
    pub input: Box<Expr>,
}

impl Negation {
    /// Type
    pub fn tpe(&self) -> SType {
        self.input.post_eval_tpe()
    }
}

impl HasStaticOpCode for Negation {
    const OP_CODE: OpCode = OpCode::NEGATION;
}

impl UnaryOp for Negation {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for Negation {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        let post_eval_tpe = input.post_eval_tpe();
        if !post_eval_tpe.is_numeric() {
            return Err(InvalidArgumentError(format!(
                "Negation: expected input type to be numeric, got {:?}",
                post_eval_tpe
            )));
        }
        Ok(Self {
            input: input.into(),
        })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Negation {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SByte,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SShort,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SLong,
                    depth: 0,
                }),
            ]
            .prop_map(|input| Self::try_build(input).unwrap())
            .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Negation>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
