use ergotree_ir::mir::constant::TryExtractFromError;
use ergotree_ir::mir::get_var::GetVar;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for GetVar {
    fn eval<'ctx>(&self, _env: &mut Env, ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(10)?; // GetVar = Fixed(10)
        match ctx.extension.values.get(&self.var_id) {
            None => Ok(Value::Opt(None)),
            Some(v) if v.tpe == self.var_tpe => Ok((Some(v.v.clone())).into()),
            Some(v) => Err(TryExtractFromError(format!(
                "GetVar: expected extension value id {} to have type {:?}, found {:?} in context extension map {}",
                self.var_id, self.var_tpe, v, ctx.extension
            ))
            .into()),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
pub mod tests {
    use super::*;
    use crate::eval::test_util::{eval_out, try_eval_out};
    use ergotree_ir::chain::context::arbitrary::DummyContextExtensionProvider;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::chain::context_extension::ContextExtension;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    const VAR_IDX: u8 = 3;
    const VAR_VAL: i32 = 123;

    /// Prepare context with single extension variable
    pub fn prepare_context() -> Context<'static> {
        let mut ctx = force_any_val::<Context>();
        ctx.extension = Box::leak(
            ContextExtension {
                values: [(VAR_IDX, VAR_VAL.into())].into_iter().collect(),
            }
            .into(),
        );
        ctx.extension_provider =
            Box::leak(DummyContextExtensionProvider(vec![ctx.extension.clone()]).into());
        ctx
    }

    /// Normal evaluation
    #[test]
    fn eval_success() {
        let ctx = prepare_context();
        let expr: Expr = GetVar {
            var_id: VAR_IDX,
            var_tpe: SType::SInt,
        }
        .into();
        let res = eval_out::<Option<i32>>(&expr, &ctx);
        assert_eq!(res, Some(VAR_VAL));
    }

    /// Variable index out of range
    #[test]
    fn eval_fail() {
        let ctx = prepare_context();
        let expr: Expr = GetVar {
            var_id: VAR_IDX + 1,
            var_tpe: SType::SInt,
        }
        .into();
        let res = eval_out::<Option<i32>>(&expr, &ctx);
        assert_eq!(res, None);
    }

    /// Variable has wrong type
    #[test]
    fn eval_wrong_type() {
        let ctx = prepare_context();
        let expr: Expr = GetVar {
            var_id: VAR_IDX,
            var_tpe: SType::SBoolean,
        }
        .into();
        let res = try_eval_out::<Value>(&expr, &ctx);
        assert!(res.is_err());
    }
}
