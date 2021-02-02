use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ByIndex {
    input: Box<Expr>,
    index: Box<Expr>,
    default: Option<Box<Expr>>,
}

impl ByIndex {
    pub const OP_CODE: OpCode = OpCode::BY_INDEX;

    pub fn tpe(&self) -> SType {
        match self.input.tpe() {
            SType::SColl(elem_tpe) => *elem_tpe,
            _ => panic!("collection is expected"),
        }
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for ByIndex {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.index.sigma_serialize(w)?;
        self.default.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?.into();
        let index = Expr::sigma_parse(r)?.into();
        let default = Option::<Box<Expr>>::sigma_parse(r)?;
        Ok(Self {
            input,
            index,
            default,
        })
    }
}

impl Evaluable for ByIndex {
    fn eval(&self, _env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::ast::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for ByIndex {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), any::<Expr>(), any::<Option<Box<Expr>>>())
                .prop_map(|(input, index, default)| Self {
                    input: input.into(),
                    index: index.into(),
                    default,
                })
                .boxed()
        }
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<ByIndex>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
