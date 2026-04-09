use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::logical_not::LogicalNot;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for LogicalNot {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(15)?; // LogicalNot = Fixed(15)
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bool = input_v.try_extract_into::<bool>()?;
        Ok((!input_v_bool).into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::eval::test_util::eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;

    fn check(input: bool) -> bool {
        let expr: Expr = LogicalNot {
            input: Expr::Const(input.into()).into(),
        }
        .into();
        eval_out_wo_ctx::<bool>(&expr)
    }

    #[test]
    fn eval() {
        assert!(!check(true));
        assert!(check(false));
    }
}
