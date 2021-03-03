use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SizeOf {
    pub input: Box<Expr>,
}

impl SizeOf {
    pub const OP_CODE: OpCode = OpCode::SIZE_OF;

    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SColl(_) => Ok(Self {
                input: input.into(),
            }),
            _ => Err(InvalidArgumentError(format!(
                "Expected SizeOf input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SInt
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for SizeOf {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        Ok(Self::new(input)?)
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::types::stype_param::STypeVar;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for SizeOf {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(SType::STypeVar(STypeVar::T).into()),
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
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<SizeOf>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
