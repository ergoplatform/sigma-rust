//! THRESHOLD composition for sigma expressions
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// THRESHOLD composition for sigma expressions
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Atleast {
    /// Number of Sigma-expression that should be proved
    pub bound: Box<Expr>,
    /// Collection of Sigma-expressions
    pub input: Box<Expr>,
}

impl Atleast {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(bound: Expr, input: Expr) -> Result<Self, InvalidArgumentError> {
        if input.post_eval_tpe() != SType::SColl(SType::SSigmaProp.into()) {
            return Err(InvalidArgumentError(format!(
                "Expected Atleast input to be SColl, got {0:?}",
                input.tpe()
            )));
        }
        if bound.post_eval_tpe() != SType::SInt {
            return Err(InvalidArgumentError(format!(
                "Atleast: expected bound type to be SInt, got {0:?}",
                bound
            )));
        }

        Ok(Self {
            bound: bound.into(),
            input: input.into(),
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SSigmaProp
    }
}

impl HasStaticOpCode for Atleast {
    const OP_CODE: OpCode = OpCode::ATLEAST;
}

impl SigmaSerializable for Atleast {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.bound.sigma_serialize(w)?;
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let bound = Expr::sigma_parse(r)?;
        let input = Expr::sigma_parse(r)?;
        Ok(Self::new(bound, input)?)
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for Atleast {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: args,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SSigmaProp.into()),
                    depth: args,
                }),
            )
                .prop_map(|(n, expr)| Self {
                    bound: n.into(),
                    input: expr.into(),
                })
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
        fn ser_roundtrip(v in any_with::<Atleast>(1)) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
