use ergotree_ir::mir::multiply_group::MultiplyGroup;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for MultiplyGroup {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let left_v = self.left.eval(env, ctx)?;
        let right_v = self.right.eval(env, ctx)?;

        match (left_v.clone(), right_v.clone()) {
            (Value::GroupElement(left), Value::GroupElement(right)) => Ok((*left * &*right).into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected MultiplyGroup input to be GroupElement, got: {0:?}",
                (left_v, right_v)
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;

    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    proptest! {

        #[test]
        fn eval_any(left in any::<EcPoint>(), right in any::<EcPoint>()) {

            let expected_mul = left.clone() * &right;

            let expr: Expr = MultiplyGroup {
                left: Box::new(Expr::Const(left.into())),
                right: Box::new(Expr::Const(right.into())),
            }
            .into();

            let ctx = Rc::new(force_any_val::<Context>());
            assert_eq!(eval_out::<EcPoint>(&expr, ctx), expected_mul);
        }
    }
}
