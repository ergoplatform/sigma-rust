//! Embedding of Boolean values to SigmaProp
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;

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
    pub(crate) const OP_CODE: OpCode = OpCode::BOOL_TO_SIGMA_PROP;

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for BoolToSigmaProp {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self {
            input: Expr::sigma_parse(r)?.into(),
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
