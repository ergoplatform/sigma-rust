use ergotree_ir::mir::multiply_group::MultiplyGroup;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for MultiplyGroup {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(40)?; // MultiplyGroup = Fixed(40)
        let left_v = self.left.eval(env, ctx)?;
        let right_v = self.right.eval(env, ctx)?;

        match (&left_v, &right_v) {
            (Value::GroupElement(left), Value::GroupElement(right)) => {
                Ok(((**left) * right).into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected MultiplyGroup input to be GroupElement, got: {0:?}",
                (left_v, right_v)
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;

    use ergo_chain_types::EcPoint;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #[test]
        fn eval_any(left in any::<EcPoint>(), right in any::<EcPoint>()) {

            let expected_mul = left * &right;

            let expr: Expr = MultiplyGroup {
                left: Box::new(Expr::Const(left.into())),
                right: Box::new(Expr::Const(right.into())),
            }
            .into();

            let ctx = force_any_val::<Context>();
            assert_eq!(eval_out::<EcPoint>(&expr, &ctx), expected_mul);
        }
    }
}
