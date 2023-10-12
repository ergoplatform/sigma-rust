use ergotree_ir::mir::coll_forall::ForAll;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ForAll {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let condition_v = self.condition.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut condition_call = |arg: Value| match &condition_v {
            Value::Lambda(func_value) => {
                let func_arg = func_value.args.first().ok_or_else(|| {
                    EvalError::NotFound(
                        "ForAll: evaluated condition has empty arguments list".to_string(),
                    )
                })?;
                env.insert(func_arg.idx, arg);
                let res = func_value.body.eval(env, ctx);
                env.remove(&func_arg.idx);
                res
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected ForAll::condition to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => {
                if *coll.elem_tpe() != self.elem_tpe {
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

    use crate::eval::tests::eval_out_wo_ctx;

    use super::*;

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
}
