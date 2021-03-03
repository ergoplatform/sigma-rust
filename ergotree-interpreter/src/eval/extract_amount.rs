use ergotree_ir::mir::extract_amount::ExtractAmount;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractAmount {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(Value::Long(ctx.ctx.box_arena.get(&b)?.value())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractAmount input to be Value::CBox, got {0:?}",
                input_v
            ))),
        }
    }
}
