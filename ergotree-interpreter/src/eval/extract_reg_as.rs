use core::convert::TryInto;

use alloc::boxed::Box;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
use ergotree_ir::mir::value::Value;
use ergotree_ir::reference::Ref;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractRegisterAs {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(50)?; // ExtractRegisterAs = Fixed(50)
        let ir_box = self
            .input
            .eval(env, ctx)?
            .try_extract_into::<Ref<'_, ErgoBox>>()?;
        let id = self.register_id.try_into().map_err(|e| {
            EvalError::RegisterIdOutOfBounds(format!(
                "register index {} is out of bounds: {:?} ",
                self.register_id, e
            ))
        })?;
        let reg_val_opt = ir_box.get_register(id).map_err(|e| {
            EvalError::NotFound(format!(
                "Error getting the register id {id} with error {e:?}"
            ))
        })?;
        match reg_val_opt {
            Some(constant) if constant.tpe == *self.elem_tpe => {
                Ok(Value::Opt(Some(Box::new(constant.v.into()))))
            }
            Some(constant) => Err(EvalError::UnexpectedValue(format!(
                "Expected register {id} to be of type {}, got {}",
                self.elem_tpe, constant.tpe
            ))),
            None => Ok(Value::Opt(None)),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::eval::test_util::{eval_out, try_eval_out};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::option_get::OptionGet;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

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
        let ctx = force_any_val::<Context>();
        let v = eval_out::<i64>(&option_get_expr, &ctx);
        assert_eq!(v, ctx.self_box.value.as_i64());
    }

    #[test]
    fn eval_box_get_reg_r0_wrong_type() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SInt.into()), // R0 (value) is long, but we're expecting int
        )
        .unwrap()
        .into();
        let option_get_expr: Expr = OptionGet::try_build(get_reg_expr).unwrap().into();
        let ctx = force_any_val::<Context>();
        assert!(try_eval_out::<Value>(&option_get_expr, &ctx).is_err());
    }
}
