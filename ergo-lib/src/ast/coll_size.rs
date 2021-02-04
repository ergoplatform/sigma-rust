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
use super::expr::InvalidArgumentError;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SizeOf {
    input: Box<Expr>,
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

impl Evaluable for SizeOf {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "SizeOf: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        Ok((normalized_input_vals.len() as i32).into())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::expr::tests::ArbExprParams;
    use crate::ast::expr::Expr;
    use crate::ast::global_vars::GlobalVars;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;
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

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<SizeOf>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }

    #[test]
    fn eval() {
        let expr: Expr = SizeOf::new(GlobalVars::Outputs.into()).unwrap().into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i32>(&expr, ctx.clone()),
            ctx.outputs.len() as i32
        );
    }
}
