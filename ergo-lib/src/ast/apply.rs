use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Apply {
    pub func: Expr,
    pub args: Vec<Expr>,
}

impl Evaluable for Apply {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let func_v = self.func.eval(env, ctx)?;
        let args_v_res: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ctx)).collect();
        let args_v = args_v_res?;
        match func_v {
            Value::FuncValue(fv) => {
                let arg_ids: Vec<u32> = fv.args.iter().map(|a| a.idx).collect();
                let mut cur_env = env.clone();
                arg_ids.iter().zip(args_v).for_each(|(idx, arg_v)| {
                    cur_env.insert(*idx, arg_v);
                });
                fv.body.eval(&cur_env, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected func_v to be Value::FuncValue got: {0:?}",
                func_v
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::bin_op::BinOp;
    use crate::ast::bin_op::LogicOp;
    use crate::ast::block::BlockValue;
    use crate::ast::func_value::*;
    use crate::ast::val_def::ValDef;
    use crate::ast::val_use::ValUse;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::*;

    #[test]
    fn user_defined_func() {
        let arg = Expr::Const(Box::new(1i32.into()));
        let bin_op = Expr::BinOp(Box::new(BinOp {
            kind: LogicOp::Eq.into(),
            left: Expr::ValUse(Box::new(ValUse {
                val_id: 1,
                tpe: SType::SInt,
            })),
            right: Expr::ValUse(Box::new(ValUse {
                val_id: 2,
                tpe: SType::SInt,
            })),
        }));
        let body = Expr::BlockValue(Box::new(BlockValue {
            items: vec![ValDef {
                id: 2,
                rhs: Expr::Const(Box::new(1i32.into())),
            }],
            result: bin_op,
        }));
        let apply: Expr = Box::new(Apply {
            func: Expr::FuncValue(Box::new(FuncValue {
                args: vec![FuncArg {
                    idx: 1,
                    tpe: SType::SInt,
                }],
                body,
            })),
            args: vec![arg],
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert!(eval_out::<bool>(&apply, ctx));
    }
}
