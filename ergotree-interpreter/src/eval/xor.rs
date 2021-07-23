use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::mir::xor::Xor;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

fn helper_xor(mut x: Vec<i8>, y: Vec<i8>) -> Vec<i8> {
    x.iter_mut().zip(y.iter()).for_each(|(x1, x2)| *x1 ^= *x2);
    x
}

impl Evaluable for Xor {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let left_v = self.left.eval(env, ctx)?;
        let right_v = self.right.eval(env, ctx)?;

        match (left_v.clone(), right_v.clone()) {
            (
                Value::Coll(CollKind::NativeColl(NativeColl::CollByte(l_byte))),
                Value::Coll(CollKind::NativeColl(NativeColl::CollByte(r_byte))),
            ) => {
                let xor = helper_xor(l_byte, r_byte);
                Ok(xor.into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Xor input to be byte array, got: {0:?}",
                (left_v, right_v)
            ))),
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn eval_1_xor_0() {
        let left = vec![1_i8];
        let right = vec![0_i8];
        let expected_xor = vec![1_i8];

        let expr: Expr = Xor {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
    }

    #[test]
    fn eval_0_xor_1() {
        let left = vec![0_i8];
        let right = vec![1_i8];
        let expected_xor = vec![1_i8];

        let expr: Expr = Xor {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
    }

    #[test]
    fn eval_1_xor_1() {
        let left = vec![1_i8];
        let right = vec![1_i8];
        let expected_xor = vec![0_i8];

        let expr: Expr = Xor {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
    }

    #[test]
    fn eval_0_xor_0() {
        let left = vec![0_i8];
        let right = vec![0_i8];
        let expected_xor = vec![0_i8];

        let expr: Expr = Xor {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
    }

    #[test]
    fn eval_1100_xor_0101() {
        let left = vec![1_i8, 1, 0, 0];
        let right = vec![0_i8, 1, 0, 1];
        let expected_xor = vec![1_i8, 0, 0, 1];

        let expr: Expr = Xor {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
    }

    proptest! {

        #[test]
        fn eval_any(left_bytes in any::<Vec<i8>>(), right_bytes in any::<Vec<i8>>()) {

            let expected_xor = helper_xor(left_bytes.clone(), right_bytes.clone());

            let expr: Expr = Xor {
                left: Box::new(Expr::Const(left_bytes.into())),
                right: Box::new(Expr::Const(right_bytes.into())),
            }
            .into();

            let ctx = Rc::new(force_any_val::<Context>());
            assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
        }
    }
}
