use alloc::vec::Vec;
use ergotree_ir::mir::apply::Apply;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::value::Value;
use hashbrown::HashMap;

use crate::eval::cost_accum::add_fixed_cost;
use crate::eval::costs;
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
                    if let Some(old_val) = env.get(*idx) {
                        existing_variables.insert(idx, old_val.clone());
                    } else {
                        new_variables.push(*idx);
                    }
                    add_fixed_cost(ctx, costs::ADD_TO_ENV_COST)?;
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
}
