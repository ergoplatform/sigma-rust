use ergotree_ir::mir::deserialize_context::DeserializeContext;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for DeserializeContext {
    fn eval(&self, _env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Err(EvalError::NotImplementedYet("DeserializeContext"))
    }
}
