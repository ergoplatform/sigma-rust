use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Selects all elements of the collection that satisfy the condition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Filter {
    /// Collection
    pub input: Box<Expr>,
    /// Function (lambda) to test each element
    pub condition: Box<Expr>,
    /// Collection element type
    pub elem_tpe: SType,
}

impl Filter {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, condition: Expr) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(*elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Map input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        match condition.tpe() {
            SType::SFunc(sfunc)
                if sfunc.t_dom == vec![input_elem_type.clone()]
                    && *sfunc.t_range == SType::SBoolean =>
            {
                Ok(Filter {
                    input: input.into(),
                    condition: condition.into(),
                    elem_tpe: input_elem_type,
                })
            }
            _ => Err(InvalidArgumentError(format!(
                "Invalid condition tpe: {0:?}",
                condition.tpe()
            ))),
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(self.elem_tpe.clone().into())
    }
}

impl HasStaticOpCode for Filter {
    const OP_CODE: OpCode = OpCode::FILTER;
}

impl SigmaSerializable for Filter {
    fn sigma_serialize<W: SigmaByteWrite>(
        &self,
        w: &mut W,
    ) -> crate::serialization::SigmaSerializeResult {
        self.input.sigma_serialize(w)?;
        self.condition.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let input = Expr::sigma_parse(r)?;
        let condition = Expr::sigma_parse(r)?;
        Ok(Filter::new(input, condition)?)
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
/// Arbitrary impl
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::types::sfunc::SFunc;
    use proptest::prelude::*;

    impl Arbitrary for Filter {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SBoolean.into()),
                    depth: 1,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SFunc(SFunc {
                        t_dom: vec![SType::SBoolean],
                        t_range: SType::SBoolean.into(),
                        tpe_params: vec![],
                    }),
                    depth: 0,
                }),
            )
                .prop_map(|(input, mapper)| Filter::new(input, mapper).unwrap())
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
        fn ser_roundtrip(v in any::<Filter>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
