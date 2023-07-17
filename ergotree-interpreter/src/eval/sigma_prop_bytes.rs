use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SigmaPropBytes {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::SigmaProp(sigma_prop) => Ok(sigma_prop.prop_bytes()?.into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected SigmaPropBytes input to be Value::SigmaProp, got {0:?}",
                input_v
            ))),
        }
    }
}

#[cfg(feature = "arbitrary")]
#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(8))]

        #[test]
        fn eval(v in any::<SigmaProp>()) {
            let expected_bytes = v.prop_bytes().unwrap();
            let input: Constant = v.into();
            let e: Expr = SigmaPropBytes {
                input: Box::new(input.into()),
            }
            .into();
            prop_assert_eq!(eval_out_wo_ctx::<Vec<u8>>(&e), expected_bytes);
        }
    }
}
