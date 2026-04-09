use alloc::vec::Vec;
use ergotree_ir::mir::extract_id::ExtractId;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractId {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(12)?; // ExtractId = Fixed(12)
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => {
                let bytes: Vec<i8> = b.box_id().into();
                Ok(bytes.into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractId input to be Value::CBox, got {0:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval() {
        let e: Expr = ExtractId {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let ctx = force_any_val::<Context>();
        let bytes: Vec<i8> = ctx.self_box.box_id().into();
        assert_eq!(eval_out::<Vec<i8>>(&e, &ctx), bytes);
    }
}
