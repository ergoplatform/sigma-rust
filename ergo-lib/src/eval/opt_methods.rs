use crate::ast::opt_methods::OptM;
use crate::ast::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for OptM {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        match self {
            OptM::Get(e) => {
                let v = e.eval(env, ctx)?;
                match v {
                    Value::Opt(opt_v) => opt_v.ok_or_else(|| {
                        EvalError::NotFound("calling Option.get on None".to_string())
                    }),
                    _ => Err(EvalError::UnexpectedExpr(format!(
                        "Don't know how to eval OptM: {0:?}",
                        self
                    ))),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::box_methods::BoxM;
    use crate::ast::box_methods::RegisterId;
    use crate::ast::expr::Expr;
    use crate::ast::global_vars::GlobalVars;
    use crate::ast::opt_methods::OptM;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    #[test]
    fn eval_get() {
        let get_reg_expr: Expr = BoxM::ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox.into()),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        }
        .into();
        let option_get_expr: Expr = Box::new(OptM::Get(get_reg_expr.into())).into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.value.as_i64());
    }
}
