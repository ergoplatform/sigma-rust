use alloc::vec::Vec;
use ergotree_ir::mir::and::And;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for And {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bools = input_v.try_extract_into::<Vec<bool>>()?;
        ctx.add_per_item_jit_cost(10, 5, 32, input_v_bools.len() as u32)?;
        Ok(input_v_bools.iter().all(|b| *b).into())
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;

    use super::*;

    use ergotree_ir::mir::expr::Expr;
    use proptest::collection;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #[test]
        fn eval(bools in collection::vec(any::<bool>(), 0..10)) {
            let expr: Expr = And {input: Expr::Const(bools.clone().into()).into()}.into();
            let ctx = force_any_val::<Context>();
            let res = eval_out::<bool>(&expr, &ctx);
            prop_assert_eq!(res, bools.iter().all(|b| *b));
        }
    }

    #[test]
    fn and_empty_jit_cost() {
        // santa eval fixture `and_empty` (tree 00960d00): And over an empty
        // Coll[Boolean] constant. Cost = Const(empty coll) 5 + And per-item at
        // n=0 (base 10 + one chunk * per_chunk 5 = 15, post n=0 chunks fix) = 20.
        // Matches the JVM under activated=3 (was 15 before the n=0 fix).
        let expr: Expr = And {
            input: Expr::Const(Vec::<bool>::new().into()).into(),
        }
        .into();
        let ctx = force_any_val::<Context>();
        let before = ctx.jit_cost_value();
        let res = eval_out::<bool>(&expr, &ctx);
        assert!(res); // all() over empty = true
        assert_eq!(ctx.jit_cost_value() - before, 20);
    }
}
