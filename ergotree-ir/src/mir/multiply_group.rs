use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Multiply two GroupElement
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MultiplyGroup {
    /// GroupElement
    pub left: Box<Expr>,
    /// GroupElement
    pub right: Box<Expr>,
}

impl MultiplyGroup {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(left: Expr, right: Expr) -> Result<Self, InvalidArgumentError> {
        match (left.tpe(), right.tpe()) {
            (SType::SGroupElement, SType::SGroupElement) => Ok(MultiplyGroup {
                left: left.into(),
                right: right.into(),
            }),
            (_, _) => Err(InvalidArgumentError(format!(
                "MultiplyGroup Expected: (SGroupElement, SGroupElement), Actual: {0:?}",
                (left.tpe(), right.tpe())
            ))),
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.left.tpe()
    }
}

impl HasStaticOpCode for MultiplyGroup {
    const OP_CODE: OpCode = OpCode::MULTIPLY_GROUP;
}

impl SigmaSerializable for MultiplyGroup {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.left.sigma_serialize(w)?;
        self.right.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let left = Expr::sigma_parse(r)?.into();
        let right = Expr::sigma_parse(r)?.into();
        Ok(MultiplyGroup { left, right })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for MultiplyGroup {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SGroupElement,
                    depth: args,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SGroupElement,
                    depth: args,
                }),
            )
                .prop_map(|(left, right)| MultiplyGroup::new(left, right).unwrap())
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
        fn ser_roundtrip(v in any::<MultiplyGroup>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
