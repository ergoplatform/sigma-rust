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

use super::constant::TryExtractInto;
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
        match self.input.post_eval_tpe() {
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
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let index_v = self.index.eval(env, ctx)?;
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "ByIndex: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        match self.default.clone() {
            Some(default) => {
                let _default_v = default.eval(env, ctx)?;
                todo!()
            }
            None => normalized_input_vals
                .get(index_v.clone().try_extract_into::<i32>()? as usize)
                .cloned()
                .ok_or_else(|| {
                    EvalError::Misc(format!(
                        "ByIndex: index {0:?} out of bounds for collection size {1:?}",
                        index_v,
                        normalized_input_vals.len()
                    ))
                }),
        }
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
