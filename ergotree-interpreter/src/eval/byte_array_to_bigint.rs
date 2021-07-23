#![allow(clippy::unwrap_used)]
use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use std::convert::TryFrom;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::EvalError::UnexpectedValue;
use crate::eval::Evaluable;

impl Evaluable for ByteArrayToBigInt {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input = self.input.eval(env, ctx)?.try_extract_into::<Vec<u8>>()?;
        if input.is_empty() {
            return Err(UnexpectedValue(
                "ByteArrayToBigInt: byte array is empty".into(),
            ));
        }
        match BigInt256::try_from(&input[..]) {
            Ok(n) => Ok(Value::BigInt(n)),
            Err(e) => Err(UnexpectedValue(e)),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::eval::tests::try_eval_out_wo_ctx;
    use num_bigint::{BigInt, Sign, ToBigInt};
    use num_traits::{Bounded, Num, Pow};

    fn eval_node(val: Vec<u8>) -> Result<BigInt256, EvalError> {
        let signed = val.iter().map(|x| *x as i8).collect::<Vec<_>>();
        let expr = ByteArrayToBigInt {
            input: Box::new(signed.into()),
        }
        .into();
        try_eval_out_wo_ctx(&expr)
    }

    #[test]
    fn eval_1() {
        let res = eval_node(vec![1]);
        assert_eq!(res, Ok(BigInt256::from_str_radix("1", 10).unwrap()));
    }

    #[test]
    fn eval_neg1() {
        let res = eval_node(vec![0xff]);
        assert_eq!(res, Ok(BigInt256::from_str_radix("-1", 10).unwrap()));
    }

    #[test]
    fn eval_neg1b() {
        let res = eval_node(vec![0xff, 0xff]);
        assert_eq!(res, Ok(BigInt256::from_str_radix("-1", 10).unwrap()));
    }

    #[test]
    fn eval_256() {
        let res = eval_node(vec![1, 0]);
        assert_eq!(res, Ok(BigInt256::from_str_radix("256", 10).unwrap()));
    }

    #[test]
    fn eval_neg32768() {
        let res = eval_node(vec![0b1000_0000, 0]);
        assert_eq!(res, Ok(BigInt256::from_str_radix("-32768", 10).unwrap()));
    }

    #[test]
    fn eval_max_bound() {
        let mut buf = vec![0xff_u8; 32];
        buf[0] = 0b0111_1111;
        match eval_node(buf) {
            Ok(n) => assert_eq!(n, BigInt256::max_value()),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn eval_min_bound() {
        let mut buf = vec![0; 32];
        buf[0] = 0b1000_0000;
        match eval_node(buf) {
            Ok(n) => assert_eq!(n, BigInt256::min_value()),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn eval_above_max_bound() {
        let mut buf = vec![0; 33];
        buf[1] = 0b1000_0000;
        // Check that value is just above MAX_BOUND
        let max_bound: BigInt = Pow::pow(BigInt::from(2), 255u32) - 1;
        assert_eq!(
            BigInt::from_bytes_be(Sign::Plus, &buf),
            max_bound + 1.to_bigint().unwrap()
        );
        let res = eval_node(buf);
        assert!(res.is_err());
    }

    #[test]
    fn eval_below_min_bound() {
        let mut buf = vec![0b1111_1111_u8; 33];
        buf[1] = 0b0111_1111;
        // Check that value is just below MIN_BOUND
        let min_bound: BigInt = -Pow::pow(BigInt::from(2), 255u32);
        assert_eq!(
            BigInt::from_signed_bytes_be(&buf),
            min_bound - 1.to_bigint().unwrap()
        );
        let res = eval_node(buf);
        assert!(res.is_err());
    }

    #[test]
    fn eval_empty() {
        let res = eval_node(Vec::new());
        assert!(res.is_err());
    }
}
