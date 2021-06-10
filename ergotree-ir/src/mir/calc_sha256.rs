use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;

/// Calc Sha 256-bit hash
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CalcSha256 {
    /// Byte array with SColl(SByte) expr type
    pub input: Box<Expr>,
}

impl CalcSha256 {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(Box::new(SType::SByte))
    }
}

impl HasStaticOpCode for CalcSha256 {
    const OP_CODE: OpCode = OpCode::CALC_SHA256;
}

impl UnaryOp for CalcSha256 {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for CalcSha256 {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SColl(Box::new(SType::SByte)))?;
        Ok(CalcSha256 {
            input: Box::new(input),
        })
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for CalcSha256 {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(Box::new(SType::SByte)),
                depth: 0,
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
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<CalcSha256>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
