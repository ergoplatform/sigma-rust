use ergotree_ir::mir::extract_bytes::ExtractBytes;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractBytes {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            // `Box.bytes` returns the retained wire slice for a parsed box (the reference impl's
            // `ErgoBox.bytes`), so a non-canonically-encoded box keeps its on-the-wire byte image
            // — unlike `bytesWithoutRef`, which re-serializes the candidate canonically.
            Value::CBox(b) => Ok(b.bytes()?.into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractBytes input to be Value::CBox, got {0:?}",
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
    use sigma_util::AsVecI8;

    #[test]
    fn eval() {
        let e: Expr = ExtractBytes {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<i8>>(&e, &ctx),
            ctx.self_box.bytes().unwrap().as_vec_i8()
        );
    }
}
