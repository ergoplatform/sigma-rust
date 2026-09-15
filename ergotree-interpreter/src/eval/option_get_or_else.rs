use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::option_get_or_else::OptionGetOrElse;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for OptionGetOrElse {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(20)?; // OptionGetOrElse = Fixed(20)
        let v = self.input.eval(env, ctx)?;
        let mut default_v = || self.default.eval(env, ctx);
        match v {
            Value::Opt(opt_v) if ctx.tree_version() >= ErgoTreeVersion::V3 => {
                opt_v.as_deref().cloned().map(Ok).unwrap_or_else(default_v)
            }
            Value::Opt(opt_v) => Ok(opt_v.as_deref().cloned().unwrap_or(default_v()?)),
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
    use super::OptionGetOrElse;
    use crate::eval::test_util::{eval_out, try_eval_out_with_version};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::mir::bin_op::{ArithOp, BinOp, BinOpKind};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
    use ergotree_ir::mir::get_var::GetVar;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval_non_empty() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let default_expr: Constant = 1i64.into();
        let option_get_expr: Expr = OptionGetOrElse::new(get_reg_expr, default_expr.into())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        let v = eval_out::<i64>(&option_get_expr, &ctx);
        assert_eq!(v, ctx.self_box.value.as_i64());
    }

    #[test]
    fn eval_empty() {
        let get_var_expr: Expr = GetVar {
            var_id: 99,
            var_tpe: SType::SLong,
        }
        .into();
        let default_expr: Constant = 1i64.into();
        let option_get_expr: Expr = OptionGetOrElse::new(get_var_expr, default_expr.into())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        let v = eval_out::<i64>(&option_get_expr, &ctx);
        assert_eq!(v, 1i64);
    }
    #[test]
    fn eval_lazy() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let divide_by_zero = Expr::BinOp(
            BinOp {
                kind: BinOpKind::Arith(ArithOp::Divide),
                left: Box::new(Constant::from(1i64).into()),
                right: Box::new(Constant::from(0i64).into()),
            }
            .into(),
        );
        let option_get_expr: Expr = OptionGetOrElse::new(get_reg_expr, divide_by_zero)
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        for tree_version in 0..ErgoTreeVersion::V3.into() {
            assert!(
                try_eval_out_with_version::<Value>(&option_get_expr, &ctx, tree_version, 3)
                    .is_err()
            );
        }
        for tree_version in ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()
        {
            assert_eq!(
                try_eval_out_with_version::<i64>(&option_get_expr, &ctx, tree_version, 3).unwrap(),
                ctx.self_box.value.as_i64()
            );
        }
    }
}
