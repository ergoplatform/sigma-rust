use ergotree_ir::mir::option_get::OptionGet;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for OptionGet {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let v = self.input.eval(env, ctx)?;
        match v {
            Value::Opt(opt_v) => {
                opt_v.ok_or_else(|| EvalError::NotFound("calling Option.get on None".to_string()))
            }
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval OptM: {0:?}",
                self
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::OptionGet;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn eval_get() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let option_get_expr: Expr = OptionGet::try_build(get_reg_expr).unwrap().into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.value());
    }
}
