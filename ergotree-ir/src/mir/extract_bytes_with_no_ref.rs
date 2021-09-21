use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Serialized box bytes without transaction_id and index
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractBytesWithNoRef {
    /// Box, type of SBox
    pub input: Box<Expr>,
}

impl ExtractBytesWithNoRef {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }
}

impl HasStaticOpCode for ExtractBytesWithNoRef {
    const OP_CODE: OpCode = OpCode::EXTRACT_BYTES_WITH_NO_REF;
}

impl OneArgOp for ExtractBytesWithNoRef {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for ExtractBytesWithNoRef {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SBox)?;
        Ok(ExtractBytesWithNoRef {
            input: input.into(),
        })
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for ExtractBytesWithNoRef {
        type Parameters = usize;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(SType::SByte.into()),
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
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractBytesWithNoRef {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
            .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
