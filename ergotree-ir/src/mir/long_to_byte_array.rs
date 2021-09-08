//! Convert SLong to byte array
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Convert SLong to byte array
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct LongToByteArray {
    /// Value of type SLong
    pub input: Box<Expr>,
}

impl LongToByteArray {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }
}

impl HasStaticOpCode for LongToByteArray {
    const OP_CODE: OpCode = OpCode::LONG_TO_BYTE_ARRAY;
}

impl OneArgOp for LongToByteArray {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for LongToByteArray {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError>
    where
        Self: Sized,
    {
        input.check_post_eval_tpe(&SType::SLong)?;
        Ok(LongToByteArray {
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

    impl Arbitrary for LongToByteArray {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SLong,
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
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<LongToByteArray>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
