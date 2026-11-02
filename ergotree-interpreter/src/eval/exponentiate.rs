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
    use ergotree_ir::unsignedbigint256::UnsignedBigInt;
    use k256::Scalar;
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

    /// Evaluate `base^exponent` through the `Exponentiate` op.
    fn eval_exponentiate(base: EcPoint, exponent: BigInt256) -> EcPoint {
        let expr: Expr = Exponentiate {
            left: Box::new(Expr::Const(base.into())),
            right: Box::new(Expr::Const(exponent.into())),
        }
        .into();
        let ctx = force_any_val::<Context>();
        eval_out::<EcPoint>(&expr, &ctx)
    }

    /// `base^k` for a non-negative `k`, computed only through the unsigned
    /// conversion — never through the signed mod-order conversion
    /// (`bigint256_to_scalar`) that the implementation itself uses, so the
    /// tests below pin the signed semantics independently.
    fn exp_unsigned(base: &EcPoint, k: UnsignedBigInt) -> EcPoint {
        let scalar: Scalar = k.into();
        ergo_chain_types::ec_point::exponentiate(base, &scalar)
    }

    // The tests below pin the reference implementation's `Exponentiate`
    // semantics (sigmastate-interpreter #731, #378): the op evaluates as
    // `CGroupElement.exp`, i.e. BouncyCastle `ECPoint.multiply` with the raw
    // signed exponent — a negative exponent gives `-(base^|k|)`, exponent 0
    // gives the identity, and an identity base stays the identity.
    // `BcDlogGroup.exponentiate` reaches the same points through an explicit
    // `exponent.mod(order)` first; both are pinned here by their results.

    #[test]
    fn eval_exponent_zero_is_identity() {
        let base = ergo_chain_types::ec_point::generator();
        assert_eq!(
            eval_exponentiate(base, BigInt256::from(0i64)),
            ergo_chain_types::ec_point::identity()
        );
    }

    #[test]
    fn eval_exponent_one_is_base() {
        let base = force_any_val::<EcPoint>();
        assert_eq!(eval_exponentiate(base, BigInt256::from(1i64)), base);
    }

    #[test]
    fn eval_identity_base_stays_identity() {
        let base = ergo_chain_types::ec_point::identity();
        assert_eq!(
            eval_exponentiate(base, BigInt256::from(-1i64)),
            ergo_chain_types::ec_point::identity()
        );
        assert_eq!(
            eval_exponentiate(base, BigInt256::from(5i64)),
            ergo_chain_types::ec_point::identity()
        );
    }

    #[test]
    fn eval_exponent_minus_one_negates_base() {
        let base = force_any_val::<EcPoint>();
        assert_eq!(eval_exponentiate(base, BigInt256::from(-1i64)), -base);
    }

    #[test]
    fn eval_exponent_large_negative() {
        // -(2^255 - 1), the smallest BigInt256 value.
        let exponent = BigInt256::from_str_radix(
            "-57896044618658097711785492504343953926634992332820282019728792003956564819967",
            10,
        )
        .unwrap();
        let base = ergo_chain_types::ec_point::generator();
        let magnitude = UnsignedBigInt::try_from(-exponent).unwrap();
        assert_eq!(
            eval_exponentiate(base, exponent),
            -exp_unsigned(&base, magnitude)
        );
    }

    proptest! {

        #[test]
        fn eval_negative_exponent_is_negated_positive(
            base in any::<EcPoint>(),
            k in 1i64..=1_000_000i64,
        ) {
            let expected = -exp_unsigned(&base, UnsignedBigInt::from(k as u64));
            prop_assert_eq!(eval_exponentiate(base, BigInt256::from(-k)), expected);
        }
    }
}
