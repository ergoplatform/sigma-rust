use ergotree_ir::mir::extract_bytes::ExtractBytes;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;

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
        ctx.add_jit_cost(12)?; // ExtractBytes = Fixed(12)
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(b.sigma_serialize_bytes()?.into()),
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
            ctx.self_box.sigma_serialize_bytes().unwrap().as_vec_i8()
        );
    }
}
