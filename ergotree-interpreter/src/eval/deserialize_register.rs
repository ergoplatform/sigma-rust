use ergotree_ir::mir::deserialize_register::DeserializeRegister;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for DeserializeRegister {
    fn eval(&self, _env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Err(EvalError::NotImplementedYet("DeserializeRegister"))
    }
}
