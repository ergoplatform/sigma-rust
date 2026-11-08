use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Tuple {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(15)?; // Tuple = Fixed(15)
        let items_v = self.items.try_mapped_ref(|i| i.eval(env, ctx));
        Ok(Value::Tup(items_v?))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;

    #[test]
    fn eval() {
        let e1: Expr = 1i64.into();
        let e2: Expr = GlobalVars::Height.into();
        let exprs = vec![e1, e2];
        let tuple: Expr = Tuple::new(exprs).unwrap().into();
        let res = eval_out_wo_ctx::<Value>(&tuple);
        assert!(matches!(res, Value::Tup(_)));
    }
}
