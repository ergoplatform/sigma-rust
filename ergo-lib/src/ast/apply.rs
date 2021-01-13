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
        match func_v {
            Value::FuncValue(fv) => {
                let args_v_res: Result<Vec<Value>, EvalError> =
                    self.args.iter().map(|arg| arg.eval(env, ctx)).collect();
                let args_v = args_v_res?;
                let arg_ids: Vec<i32> = fv.args.iter().map(|a| a.idx).collect();
                let mut cur_env = env.clone();
                arg_ids.iter().zip(args_v).for_each(|(idx, arg_v)| {
                    cur_env.insert(*idx, arg_v);
                });
                fv.body.eval(&cur_env, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected fold_op to be Value::FuncValue got: {0:?}",
                func_v
            ))),
        }
    }
}
