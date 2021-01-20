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
            Expr::CalcBlake2b256(op) => op.eval(env, ctx),
            Expr::Fold(op) => op.eval(env, ctx),
            Expr::ExtractRegisterAs(op) => op.eval(env, ctx),
            Expr::GlobalVars(op) => op.eval(env, ctx),
            Expr::MethodCall(op) => op.eval(env, ctx),
            Expr::ProperyCall(op) => op.eval(env, ctx),
            Expr::BinOp(op) => op.eval(env, ctx),
            Expr::Context => Ok(Value::Context(ctx.ctx.clone())),
            Expr::OptionGet(v) => v.eval(env, ctx),
            Expr::Apply(op) => op.eval(env, ctx),
            Expr::FuncValue(op) => op.eval(env, ctx),
            Expr::ValUse(op) => op.eval(env, ctx),
            Expr::BlockValue(op) => op.eval(env, ctx),
            Expr::SelectField(op) => op.eval(env, ctx),
            Expr::ExtractAmount(op) => op.eval(env, ctx),
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval Expr: {0:?}",
                self
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::expr::tests::*;
    use crate::eval::context::Context;
    use crate::eval::cost_accum::CostAccumulator;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::*;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn eval(e in any_with::<Expr>(ArbExprParams{tpe: SType::SBoolean, depth: 4})) {
            dbg!(&e);
            let ctx = Rc::new(force_any_val::<Context>());
            let cost_accum = CostAccumulator::new(0, None);
            let mut ectx = EvalContext::new(ctx, cost_accum);
            let r = e.eval(&Env::empty(), &mut ectx);
            prop_assert!(r.is_ok());
        }
    }
}
