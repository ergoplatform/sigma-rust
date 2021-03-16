use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for CreateProveDlog {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let value_v = self.value.eval(env, ctx)?;
        match value_v {
            Value::GroupElement(ecpoint) => {
                let prove_dlog = ProveDlog::new(*ecpoint);
                Ok(prove_dlog.into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected CreateProveDlog input to be Value::GroupElement, got {0:?}",
                value_v
            ))),
        }
    }
}
