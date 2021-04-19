// use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::get_var::GetVar;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for GetVar {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        todo!()
    }
}
