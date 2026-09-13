use alloc::vec::Vec;
use ergotree_ir::mir::coll_forall::ForAll;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ForAll {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let condition_v = self.condition.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut condition_call = |arg: Value<'ctx>| match &condition_v {
            Value::Lambda(func_value) => crate::eval::eval_lambda_1arg(
                func_value,
                arg,
                env,
                ctx,
                "ForAll: evaluated condition has empty arguments list",
            ),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected ForAll::condition to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => {
                if coll.elem_tpe() != &*self.elem_tpe {
                    return Err(EvalError::UnexpectedValue(format!(
                        "expected ForAll input element type to be {0:?}, got: {1:?}",
                        self.elem_tpe,
                        coll.elem_tpe()
                    )));
                };
                Ok(coll.as_vec())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Map input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        ctx.add_per_item_jit_cost(3, 1, 10, normalized_input_vals.len() as u32)?;

        for item in normalized_input_vals {
            let res = condition_call(item)?.try_extract_into::<bool>()?;
            if !res {
                return Ok(false.into());
            }
        }
        Ok(true.into())
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use crate::eval::test_util::eval_out_wo_ctx;

    use super::*;

    use alloc::boxed::Box;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::stype::SType;

    fn check(coll: Vec<i32>) {
        let body: Expr = BinOp {
            kind: RelationOp::Le.into(),
            left: Box::new(Expr::Const(1i32.into())),
            right: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
        }
        .into();
        let expr: Expr = ForAll::new(
            coll.clone().into(),
            FuncValue::new(
                vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SInt,
                }],
                body,
            )
            .into(),
        )
        .unwrap()
        .into();
        assert_eq!(
            eval_out_wo_ctx::<bool>(&expr),
            coll.iter().all(|it| 1 <= *it)
        );
    }

    #[test]
    fn eval_emty_coll() {
        check(Vec::<i32>::new());
    }

    #[test]
    fn eval_true() {
        check(vec![1, 1]);
    }

    #[test]
    fn eval_false() {
        check(vec![1, 2]);
    }

    // Bug 6 regression: each lambda invocation inside ForAll (and Map/Filter/
    // Fold/Exists) must charge ADD_TO_ENV_COST (5 JIT) — matching Scala's
    // per-iteration env binding cost. Pre-fix, coll ops only paid the collection
    // base/chunk cost, underpricing scripts that rely on large collections.
    // Short-circuit correctness is verified by the 4-iter vs 1-iter delta.
    #[test]
    fn forall_charges_add_to_env_per_iteration() {
        use crate::eval::test_util::try_eval_out;
        use ergotree_ir::chain::context::Context;
        use sigma_test_util::force_any_val;

        let run = |coll: Vec<i32>, predicate_const: bool| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let body: Expr = Expr::Const(predicate_const.into());
            let expr: Expr = ForAll::new(
                coll.into(),
                FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::SInt,
                    }],
                    body,
                )
                .into(),
            )
            .unwrap()
            .into();
            let _: bool = try_eval_out(&expr, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        // All-true predicate → 4 lambda invocations (no short-circuit).
        let delta_full = run(vec![1, 2, 3, 4], true);
        // All-false predicate → ForAll short-circuits on item 0, 1 invocation.
        let delta_short = run(vec![1, 2, 3, 4], false);

        // Per invocation: ADD_TO_ENV_COST (5) + Const body eval (5) = 10 JIT.
        // Full-eval runs 3 more iterations than short-circuit, so delta is
        // 3 × 10 = 30. Pre-fix only body charged per iteration → delta 3 × 5 = 15.
        assert_eq!(
            delta_full - delta_short,
            30,
            "ForAll must charge ADD_TO_ENV_COST (5) per lambda invocation on \
             top of the Const body (5). Got {} JIT delta between full-eval \
             (4 iters) and short-circuit (1 iter); expected 30.",
            delta_full - delta_short,
        );
    }
}
