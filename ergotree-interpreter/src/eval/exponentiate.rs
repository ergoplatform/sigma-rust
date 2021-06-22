use ergotree_ir::mir::exponentiate::Exponentiate;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::dlog_group;
use k256::Scalar;
use num_bigint::{BigInt, BigUint};

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Exponentiate {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let left_v = self.left.eval(env, ctx)?;
        let right_v = self.right.eval(env, ctx)?;

        let right_bui: Option<BigUint> = match right_v.clone() {
            Value::BigInt(bi) => match bi.sign() {
                num_bigint::Sign::Minus => {
                    return Err(EvalError::UnexpectedValue(format!(
                        "Expected Exponentiate op exponent to be positive, got: {0:?}",
                        bi
                    )))
                }
                _ => BigInt::to_biguint(&bi),
            },
            _ => None,
        };

        let right_scalar: Option<Scalar> = match right_bui {
            Some(bui) => dlog_group::from_biguint(bui),
            _ => None,
        };

        match (left_v.clone(), right_scalar) {
            (Value::GroupElement(group), Some(exp)) => {
                Ok(dlog_group::exponentiate(&group, &exp).into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "Exponentiate input should be GroupElement, BigInt (<= 256 bit). Received: {0:?}",
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
    use crate::eval::tests::{eval_out, try_eval_out};
    use crate::sigma_protocol::private_input::DlogProverInput;

    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use num_bigint::{BigInt, Sign};
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    proptest! {

        #[test]
        fn eval_any(left in any::<EcPoint>(), pi in any::<DlogProverInput>()) {

            let r_g_array = pi.w.to_bytes();
            let r_b_array: &[u8] = r_g_array.as_slice();

            let right: BigInt = BigInt::from_bytes_be(Sign::Plus, r_b_array);

            let expected_exp = dlog_group::exponentiate(
                &left,
                &dlog_group::from_biguint(BigInt::to_biguint(&right).unwrap()).unwrap()
            );

            let expr: Expr = Exponentiate {
                left: Box::new(Expr::Const(left.into())),
                right: Box::new(Expr::Const(right.into())),
            }
            .into();

            let ctx = Rc::new(force_any_val::<Context>());
            assert_eq!(eval_out::<EcPoint>(&expr, ctx), expected_exp);
        }
    }

    #[test]
    fn eval_exponent_negative() {
        let left = force_any_val::<EcPoint>();
        let right = BigInt::parse_bytes(b"-1", 10).unwrap();
        let expr: Expr = Exponentiate {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert!(try_eval_out::<EcPoint>(&expr, ctx).is_err());
    }

    #[test]
    fn eval_exponent_greater_than_256_bit() {
        let left = force_any_val::<EcPoint>();
        let right = BigInt::parse_bytes
            (b"2240553423075396383515373673723462837462821468959827293462897346923874293642946928374923875928657983456938759287459236459287459238759346592837", 10)
            .unwrap();
        let expr: Expr = Exponentiate {
            left: Box::new(Expr::Const(left.into())),
            right: Box::new(Expr::Const(right.into())),
        }
        .into();

        let ctx = Rc::new(force_any_val::<Context>());
        assert!(try_eval_out::<EcPoint>(&expr, ctx).is_err());
    }
}
