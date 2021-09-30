use std::rc::Rc;

use ergotree_ir::ir_ergo_box::IrErgoBox;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractRegisterAs {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let ir_box = self
            .input
            .eval(env, ctx)?
            .try_extract_into::<Rc<dyn IrErgoBox>>()?;
        Ok(Value::Opt(Box::new(
            ir_box
                .get_register(self.register_id)
                .map(|c| Value::from(c.v)),
        )))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::option_get::OptionGet;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn eval_box_get_reg_r0() {
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
