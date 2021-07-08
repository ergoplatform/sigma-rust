use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Get collection element by index
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ByIndex {
    /// Collection
    pub input: Box<Expr>,
    /// Element index
    pub index: Box<Expr>,
    /// Default value, returned if index is out of bounds in "Coll.getOrElse()" op
    pub default: Option<Box<Expr>>,
    /// Input collection element type
    input_elem_tpe: SType,
}

impl ByIndex {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(
        input: Expr,
        index: Expr,
        default: Option<Box<Expr>>,
    ) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(*elem_type.clone()),
            _ => Err(InvalidArgumentError(format!(
                "Expected ByIndex input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        if index.post_eval_tpe() != SType::SInt {
            return Err(InvalidArgumentError(format!(
                "ByIndex: expected index type to be SInt, got {0:?}",
                index
            )));
        }
        if !default
            .clone()
            .map(|expr| expr.post_eval_tpe() == input_elem_type)
            .unwrap_or(true)
        {
            return Err(InvalidArgumentError(format!(
                "ByIndex: expected default type to be {0:?}, got {1:?}",
                input_elem_type, default
            )));
        }
        Ok(Self {
            input: input.into(),
            index: index.into(),
            default,
            input_elem_tpe: input_elem_type,
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.input_elem_tpe.clone()
    }
}

impl HasStaticOpCode for ByIndex {
    const OP_CODE: OpCode = OpCode::BY_INDEX;
}

impl SigmaSerializable for ByIndex {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.index.sigma_serialize(w)?;
        self.default.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let index = Expr::sigma_parse(r)?;
        let default = Option::<Box<Expr>>::sigma_parse(r)?;
        Ok(Self::new(input, index, default)?)
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
/// Arbitrary impl
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;
    use proptest::result::Probability;

    impl Arbitrary for ByIndex {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ArbExprParams;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(args.tpe.clone().into()),
                    depth: args.depth,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Option<Box<Expr>>>((
                    Probability::default(),
                    ArbExprParams {
                        tpe: args.tpe,
                        depth: 0,
                    },
                )),
            )
                .prop_map(|(input, index, default)| Self::new(input, index, default).unwrap())
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<ByIndex>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
