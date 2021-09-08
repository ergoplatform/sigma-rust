use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SizeOf {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "SizeOf: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        Ok((normalized_input_vals.len() as i32).into())
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn eval() {
        let expr: Expr = SizeOf::try_build(GlobalVars::Outputs.into())
            .unwrap()
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i32>(&expr, ctx.clone()),
            ctx.outputs.len() as i32
        );
    }
}
