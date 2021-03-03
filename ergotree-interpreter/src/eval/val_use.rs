use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ValUse {
    fn eval(&self, env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        env.get(self.val_id).cloned().ok_or_else(|| {
            EvalError::NotFound(format!("no value in env for id: {0:?}", self.val_id))
        })
    }
}
