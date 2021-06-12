use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Selects an interval of elements
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Slice {
    /// Collection
    pub input: Box<Expr>,
    /// The lowest index to include from this collection
    pub from: Box<Expr>,
    /// The lowest index to exclude from this collection
    pub until: Box<Expr>,
}

impl Slice {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, from: Expr, until: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SColl(_) => {}
            _ => {
                return Err(InvalidArgumentError(format!(
                    "Expected Slice input to be SColl, got {0:?}",
                    input.tpe()
                )))
            }
        };
        if from.post_eval_tpe() != SType::SInt {
            return Err(InvalidArgumentError(format!(
                "Slice: expected from type to be SInt, got {0:?}",
                from
            )));
        }
        if until.post_eval_tpe() != SType::SInt {
            return Err(InvalidArgumentError(format!(
                "Slice: expected until type to be SInt, got {0:?}",
                until
            )));
        }
        Ok(Self {
            input: input.into(),
            from: from.into(),
            until: until.into(),
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.input.tpe()
    }
}

impl HasStaticOpCode for Slice {
    const OP_CODE: OpCode = OpCode::SLICE;
}

impl SigmaSerializable for Slice {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.from.sigma_serialize(w)?;
        self.until.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let from = Expr::sigma_parse(r)?;
        let until = Expr::sigma_parse(r)?;
        Ok(Self::new(input, from, until)?)
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for Slice {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SBoolean.into()),
                    depth: 1,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
            )
                .prop_map(|(input, from, until)| Self::new(input, from, until).unwrap())
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
        fn ser_roundtrip(v in any::<Slice>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
