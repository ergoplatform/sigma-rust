use ergo_chain_types::EcPoint;
use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::exponentiate::Exponentiate;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::dlog_group;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Exponentiate {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(900)?; // Exponentiate = Fixed(900)
        let left_v = self.left.eval(env, ctx)?.try_extract_into()?;
        let right_v = self.right.eval(env, ctx)?.try_extract_into()?;

        exponentiate(left_v, right_v)
    }
}

pub(crate) fn exponentiate(
    base: EcPoint,
    exponent: BigInt256,
) -> Result<Value<'static>, EvalError> {
    let exp_scalar = dlog_group::bigint256_to_scalar(exponent);
    Ok(ergo_chain_types::ec_point::exponentiate(&base, &exp_scalar).into())
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use crate::sigma_protocol::private_input::DlogProverInput;
    use ergotree_ir::chain::context::Context;

    use ergo_chain_types::EcPoint;
    use ergotree_ir::bigint256::BigInt256;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::sigma_protocol::dlog_group::scalar_to_bigint256;
    use num_traits::Num;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #[test]
        fn eval_any(left in any::<EcPoint>(), pi in any::<DlogProverInput>()) {
            // Shift right to make sure that the MSB is 0, so that the Scalar
            // can be converted to a BigInt256 and back
            let right: BigInt256 = scalar_to_bigint256(pi.w.as_scalar_ref() >> 1).unwrap();

            let expected_exp = ergo_chain_types::ec_point::exponentiate(
                &left,
                &dlog_group::bigint256_to_scalar(right)
            );

            let expr: Expr = Exponentiate {
                left: Box::new(Expr::Const(left.into())),
                right: Box::new(Expr::Const(right.into())),
            }
            .into();

            let ctx = force_any_val::<Context>();
            assert_eq!(eval_out::<EcPoint>(&expr, &ctx), expected_exp);
        }
    }

    #[test]
    fn eval_exponent_negative() {
        let left = force_any_val::<EcPoint>();
        let right = BigInt256::from_str_radix("-1", 10).unwrap();

        let expected_exp = ergo_chain_types::ec_point::exponentiate(
            &left,
            &dlog_group::bigint256_to_scalar(right),
        );

        let expr: Expr = Exponentiate {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<EcPoint>(&expr, &ctx), expected_exp);
    }
}
