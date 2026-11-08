use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractScriptBytes {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(10)?; // ExtractScriptBytes = Fixed(10)
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(b.script_bytes()?.into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractScriptBytes input to be Value::CBox, got {0:?}",
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
        let e: Expr = ExtractScriptBytes {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<i8>>(&e, &ctx),
            ctx.self_box.script_bytes().unwrap()
        );
    }
}
