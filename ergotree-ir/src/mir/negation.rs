use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Negation operation on numeric type.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Negation {
    /// Input expr of numeric type
    pub input: Box<Expr>,
}

impl Negation {
    pub(crate) const OP_CODE: OpCode = OpCode::NEGATION;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SByte | SType::SShort | SType::SInt | SType::SLong  => Ok(Self {
                input: input.into(),
            }),
            tpe => Err(InvalidArgumentError(format!(
                "Negation: expected input type to be numeric, got {:?}",
                tpe
            ))),
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.input.post_eval_tpe()
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for Negation {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self::new(Expr::sigma_parse(r)?)?)
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Negation {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SByte,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SShort,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SLong,
                    depth: 0,
                }),
            ]
            .prop_map(|input| Self::new(input).unwrap())
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
        fn ser_roundtrip(v in any::<Negation>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
