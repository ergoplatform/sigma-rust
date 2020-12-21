use crate::ast::expr::Expr;
use crate::ast::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for Expr {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        ctx.cost_accum.add_cost_of(self)?;
        match self {
            Expr::Const(c) => Ok(c.v.clone()),
            Expr::PredefFunc(_) => todo!(),
            Expr::Fold(_) => todo!(),
            Expr::ExtractRegisterAs(op) => op.eval(env, ctx),
            Expr::GlobalVars(op) => op.eval(env, ctx),
            Expr::MethodCall(op) => op.eval(env, ctx),
            Expr::ProperyCall(op) => op.eval(env, ctx),
            Expr::BinOp(op) => op.eval(env, ctx),
            Expr::Context => Ok(Value::Context(ctx.ctx.clone())),
            Expr::OptionGet(v) => v.eval(env, ctx),
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval Expr: {}",
                self
            ))),
        }
    }
}
