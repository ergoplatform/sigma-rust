use crate::ast::expr::Expr;
use crate::ast::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for Expr {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        ectx.cost_accum.add_cost_of(self)?;
        match self {
            Expr::Const(c) => Ok(c.v.clone()),
            Expr::PredefFunc(_) => todo!(),
            Expr::Fold(_) => todo!(),
            Expr::ExtractRegisterAs(v) => v.eval(env, ectx),
            Expr::GlobalVars(v) => v.eval(env, ectx),
            Expr::MethodCall(v) => v.eval(env, ectx),
            Expr::ProperyCall(v) => v.eval(env, ectx),
            Expr::BinOp(_bin_op, _l, _r) => {
                todo!()
                // let _v_l = eval(l, env, ca, ctx)?;
                // let _v_r = eval(r, env, ca, ctx)?;
                // ca.add_cost_of(expr);
                // Ok(match bin_op {
                //     BinOp::Num(op) => match op {
                //         NumOp::Add => v_l + v_r,
                //     },
                // })
            }
            Expr::Context => Ok(Value::Context(ectx.ctx.clone())),
            Expr::OptionGet(v) => v.eval(env, ectx),
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval Expr: {}",
                self
            ))),
        }
    }
}
