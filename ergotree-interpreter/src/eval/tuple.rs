use std::convert::TryInto;

use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Tuple {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let items_v: Result<Vec<Value>, EvalError> =
            self.items.iter().map(|i| i.eval(env, ctx)).collect();
        Ok(Value::Tup(items_v?.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
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
