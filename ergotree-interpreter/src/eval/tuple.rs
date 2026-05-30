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
        // sigma-state restricts tuples to exactly 2 elements: `Tuple.eval`
        // rejects `items.length != 2` with "Invalid tuple" before doing any work
        // (sigma `values.scala`: "in v5.0 version we support only tuples of 2
        // elements to be equivalent with v4.x"). The check is unconditional —
        // no version gate, and sigma-state 6.0.3 evaluates every height through
        // it. sigma-rust models tuples as flat N-ary (`TupleItems =
        // BoundedVec<2,255>`), so without this guard it accepts arity>=3 tuples
        // the JVM rejects — a consensus accept/reject divergence (sigma-rust
        // would accept a spend the JVM rejects). Mirror the JVM: reject non-pairs.
        if self.items.len() != 2 {
            return Err(EvalError::Misc(format!(
                "Tuple: sigma-state allows only 2-element tuples, got {}",
                self.items.len()
            )));
        }
        let items_v = self.items.try_mapped_ref(|i| i.eval(env, ctx));
        Ok(Value::Tup(items_v?))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval() {
        let e1: Expr = 1i64.into();
        let e2: Expr = GlobalVars::Height.into();
        let exprs = vec![e1, e2];
        let tuple: Expr = Tuple::new(exprs).unwrap().into();
        let res = eval_out_wo_ctx::<Value>(&tuple);
        assert!(matches!(res, Value::Tup(_)));
    }

    #[test]
    fn eval_rejects_non_pair_tuple() {
        // sigma-state rejects tuples with length != 2 ("Invalid tuple", JVM
        // consensus). A flat arity-3 tuple is constructible here (TupleItems
        // allows 2..=255), but must be rejected at eval to match the JVM — else
        // sigma-rust would accept a spend the JVM rejects.
        let e1: Expr = 1i64.into();
        let e2: Expr = 2i64.into();
        let e3: Expr = 3i64.into();
        let tuple: Expr = Tuple::new(vec![e1, e2, e3]).unwrap().into();
        let ctx = force_any_val::<Context>();
        let res = try_eval_out::<Value>(&tuple, &ctx);
        assert!(
            res.is_err(),
            "arity-3 Tuple must be rejected at eval, got {res:?}"
        );
    }
}
