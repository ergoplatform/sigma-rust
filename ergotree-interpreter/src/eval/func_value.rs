use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::value::Lambda;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for FuncValue {
    fn eval(&self, _env: &mut Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(Value::Lambda(Lambda {
            args: self.args().to_vec(),
            body: self.body().clone().into(),
        }))
    }
}
