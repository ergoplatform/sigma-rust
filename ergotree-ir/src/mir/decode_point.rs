//! Decode byte array to EC point

use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Decode byte array to EC point
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DecodePoint {
    /// Byte array to be decoded
    pub input: Box<Expr>,
}

impl DecodePoint {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SGroupElement
    }
}

impl HasStaticOpCode for DecodePoint {
    const OP_CODE: OpCode = OpCode::DECODE_POINT;
}

impl UnaryOp for DecodePoint {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for DecodePoint {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SColl(Box::new(SType::SByte)))?;
        Ok(Self {
            input: input.into(),
        })
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for DecodePoint {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(SType::SByte.into()),
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
        fn ser_roundtrip(v in any_with::<DecodePoint>(1)) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
