use alloc::vec::Vec;
use ergotree_ir::mir::apply::Apply;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::value::Value;
use hashbrown::HashMap;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Apply {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(30)?; // Apply = Fixed(30)
        let func_v: Value<'ctx> = self.func.eval(env, ctx)?;
        let args_v: Vec<Value> = self
            .args
            .iter()
            .map(|arg| arg.eval(env, ctx))
            .collect::<Result<_, EvalError>>()?;
        match func_v {
            Value::Lambda(fv) => {
                let arg_ids: Vec<ValId> = fv.args.iter().map(|a| a.idx).collect();
                let mut existing_variables = HashMap::new();
                let mut new_variables = vec![];
                for (idx, arg_v) in arg_ids.iter().zip(args_v) {
                    ctx.add_jit_cost(crate::eval::ADD_TO_ENV_COST)?;
                    if let Some(old_val) = env.get(*idx) {
                        existing_variables.insert(idx, old_val.clone());
                    } else {
                        new_variables.push(*idx);
                    }
                    env.insert(*idx, arg_v);
                }
                let res = fv.body.eval(env, ctx);
                new_variables.into_iter().for_each(|idx| {
                    env.remove(&idx);
                });
                existing_variables
                    .into_iter()
                    .for_each(|(idx, orig_value)| {
                        env.insert(*idx, orig_value);
                    });

                res
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected func_v to be Value::FuncValue got: {0:?}",
                func_v
            ))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use alloc::boxed::Box;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::val_def::ValDef;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::stype::SType;

    use crate::eval::test_util::eval_out_wo_ctx;

    use super::*;

    #[test]
    fn eval_user_defined_func_call() {
        let arg = Expr::Const(1i32.into());
        let bin_op = Expr::BinOp(
            BinOp {
                kind: RelationOp::Eq.into(),
                left: Box::new(
                    ValUse {
                        val_id: 1.into(),
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
                right: Box::new(
                    ValUse {
                        val_id: 2.into(),
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
            }
            .into(),
        );
        let body = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: 2.into(),
                    rhs: Box::new(Expr::Const(1i32.into())),
                }
                .into()],
                result: Box::new(bin_op),
            }
            .into(),
        );
        let apply: Expr = Apply::new(
            FuncValue::new(
                vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SInt,
                }],
                body,
            )
            .into(),
            vec![arg],
        )
        .unwrap()
        .into();
        assert!(eval_out_wo_ctx::<bool>(&apply));
    }

    // Parity gap: Scala charges ADD_TO_ENV_COST (5 JIT) on every env
    // insertion. Apply binds each lambda arg into the interpreter env, so an
    // N-arg application must pay 5×N ADD_TO_ENV on top of Apply=Fixed(30),
    // FuncValue=Fixed(5), and the per-arg value eval. Pre-fix the binding
    // loop charged 0, undercharging every user-lambda application by 5×N.
    #[test]
    fn apply_charges_add_to_env_per_arg_binding() {
        use crate::eval::test_util::try_eval_out;
        use ergotree_ir::chain::context::Context;
        use sigma_test_util::force_any_val;

        // Apply a FuncValue of `n_args` Int params (body ignores them and
        // returns a Bool const) to `n_args` Int constants; return JIT cost.
        let run = |n_args: u32| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let args: alloc::vec::Vec<FuncArg> = (1..=n_args)
                .map(|i| FuncArg {
                    idx: i.into(),
                    tpe: SType::SInt,
                })
                .collect();
            let arg_exprs: alloc::vec::Vec<Expr> =
                (1..=n_args).map(|_| Expr::Const(1i32.into())).collect();
            let apply: Expr = Apply::new(
                FuncValue::new(args, Expr::Const(true.into())).into(),
                arg_exprs,
            )
            .unwrap()
            .into();
            let _: bool = try_eval_out(&apply, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        // Each extra arg adds: arg Const eval (5) + ADD_TO_ENV_COST (5) = 10.
        // Apply (30), FuncValue (5) and the body Const (5) are identical
        // across both runs, so they cancel in the delta.
        let delta_1 = run(1);
        let delta_2 = run(2);
        assert_eq!(
            delta_2 - delta_1,
            10,
            "Apply must charge ADD_TO_ENV_COST (5 JIT) per lambda-arg binding \
             on top of the per-arg value eval (5); got {} JIT delta between a \
             2-arg and 1-arg application, expected 10 (pre-fix would be 5).",
            delta_2 - delta_1,
        );
    }
}
