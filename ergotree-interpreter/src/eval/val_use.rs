use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ValUse {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        _ctx.add_jit_cost(5)?; // ValUse = Fixed(5)
        env.get(self.val_id).cloned().ok_or_else(|| {
            EvalError::NotFound(format!("no value in env for id: {0:?}", self.val_id))
        })
    }
}
