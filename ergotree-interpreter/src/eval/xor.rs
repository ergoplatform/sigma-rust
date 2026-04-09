use alloc::sync::Arc;

use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::mir::xor::Xor;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

fn helper_xor(x: &[i8], y: &[i8]) -> Arc<[i8]> {
    x.iter().zip(y.iter()).map(|(x1, x2)| *x1 ^ *x2).collect()
}

impl Evaluable for Xor {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let left_v = self.left.eval(env, ctx)?;
        let right_v = self.right.eval(env, ctx)?;

        match (left_v.clone(), right_v.clone()) {
            (
                Value::Coll(CollKind::NativeColl(NativeColl::CollByte(l_byte))),
                Value::Coll(CollKind::NativeColl(NativeColl::CollByte(r_byte))),
            ) => {
                ctx.add_per_item_jit_cost(10, 2, 128, l_byte.len() as u32)?;
                let xor = helper_xor(&l_byte, &r_byte);
                Ok(CollKind::NativeColl(NativeColl::CollByte(xor)).into())
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
    use crate::eval::test_util::{eval_out, eval_out_wo_ctx};
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

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

        assert_eq!(eval_out_wo_ctx::<Vec<i8>>(&expr), expected_xor);
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

        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<Vec<i8>>(&expr, &ctx), expected_xor);
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

        assert_eq!(eval_out_wo_ctx::<Vec<i8>>(&expr), expected_xor);
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

        assert_eq!(eval_out_wo_ctx::<Vec<i8>>(&expr), expected_xor);
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

        assert_eq!(eval_out_wo_ctx::<Vec<i8>>(&expr), expected_xor);
    }

    proptest! {

        #[test]
        fn eval_any(left_bytes in any::<Vec<i8>>(), right_bytes in any::<Vec<i8>>()) {

            let expected_xor = helper_xor(&left_bytes, &right_bytes);

            let expr: Expr = Xor {
                left: Box::new(Expr::Const(left_bytes.into())),
                right: Box::new(Expr::Const(right_bytes.into())),
            }
            .into();

            assert_eq!(&eval_out_wo_ctx::<Vec<i8>>(&expr)[..], &expected_xor[..]);
        }
    }
}
