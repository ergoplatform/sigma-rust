use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_box::RegisterId;
use crate::eval::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::types::stype::SType;

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractRegisterAs {
    /// Box
    pub input: Box<Expr>,
    /// Register id to extract value from
    pub register_id: RegisterId,
    /// Type
    pub tpe: SType,
}

impl Evaluable for ExtractRegisterAs {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(Value::Opt(Box::new(
            self.input
                .eval(env, ctx)?
                .try_extract_into::<ErgoBox>()?
                .get_register(self.register_id)
                .map(|c| c.v),
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::global_vars::GlobalVars;
    use crate::ast::option_get::OptionGet;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_box_get_reg() {
        let get_reg_expr: Expr = Box::new(ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox.into()),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        })
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
