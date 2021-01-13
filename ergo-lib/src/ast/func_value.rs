use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncArg {
    pub idx: u32,
    pub tpe: SType,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncValue {
    pub args: Vec<FuncArg>,
    pub body: Expr,
}

impl Evaluable for FuncValue {
    fn eval(&self, _env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(Value::FuncValue(Box::new(self.clone())))
    }
}
