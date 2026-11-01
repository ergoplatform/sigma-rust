use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::value::Lambda;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for FuncValue {
    fn eval<'ctx>(&self, _env: &mut Env, _ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        Ok(Value::Lambda(Lambda {
            args: self.args().to_vec(),
            body: self.body().clone().into(),
        }))
    }
}
