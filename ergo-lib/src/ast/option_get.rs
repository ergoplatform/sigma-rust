use super::expr::Expr;
use crate::ast::value::Value;
use crate::eval::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OptionGet {
    pub input: Expr,
}

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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::box_methods::BoxM;
    use crate::ast::box_methods::RegisterId;
    use crate::ast::expr::Expr;
    use crate::ast::global_vars::GlobalVars;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::OptionGet;

    #[test]
    fn eval_get() {
        let get_reg_expr: Expr = BoxM::ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox.into()),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        }
        .into();
        let option_get_expr: Expr = Box::new(OptionGet {
            input: get_reg_expr,
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.value.as_i64());
    }
}
