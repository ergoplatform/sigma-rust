use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractAmount {
    input: Box<Expr>,
}

impl ExtractAmount {
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        if input.tpe() == SType::SBox {
            Ok(ExtractAmount {
                input: input.into(),
            })
        } else {
            Err(InvalidArgumentError(format!(
                "ExtractAmount type expected to be SBox, got {0:?}",
                input.tpe()
            )))
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SBox
    }
}

impl Evaluable for ExtractAmount {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(Value::Long(b.value.as_i64())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractAmount input to be Value::CBox, got {0:?}",
                input_v
            ))),
        }
    }
}
