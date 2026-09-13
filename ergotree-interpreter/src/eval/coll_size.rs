use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SizeOf {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(14)?; // SizeOf = Fixed(14)
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::Coll(coll) => Ok((coll.len() as i32).into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "SizeOf: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval() {
        let expr: Expr = SizeOf::try_build(GlobalVars::Outputs.into())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<i32>(&expr, &ctx), ctx.outputs.len() as i32);
    }
}
