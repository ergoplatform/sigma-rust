use ergotree_ir::mir::option_is_defined::OptionIsDefined;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for OptionIsDefined {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(10)?; // OptionIsDefined = Fixed(10)
        let v = self.input.eval(env, ctx)?;
        match v {
            Value::Opt(opt_v) => Ok(opt_v.is_some().into()),
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval OptM: {0:?}",
                self
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::OptionIsDefined;
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let option_expr: Expr = OptionIsDefined {
            input: Box::new(get_reg_expr),
        }
        .into();
        let ctx = force_any_val::<Context>();
        let v = eval_out::<bool>(&option_expr, &ctx);
        // R0 is always defined (box value)
        assert!(v);
    }
}
