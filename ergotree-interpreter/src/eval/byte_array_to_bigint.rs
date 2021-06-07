#![allow(clippy::unwrap_used)]
use ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt;
use ergotree_ir::mir::value::Value;

use crate::eval::bigint::{MAX_BOUND, MIN_BOUND};
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::EvalError::UnexpectedValue;
use crate::eval::Evaluable;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value::BigInt;

impl Evaluable for ByteArrayToBigInt {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input = self.input.eval(env, ctx)?.try_extract_into::<Vec<u8>>()?;
        if input.is_empty() {
            return Err(UnexpectedValue(
                "ByteArrayToBigInt: byte array is empty".into(),
            ));
        }
        let n = num_bigint::BigInt::from_signed_bytes_be(&input);
        if n > *MAX_BOUND {
            return Err(UnexpectedValue(
                "ByteArrayToBigInt: byte array encodes number larger than 2^255-1".into(),
            ));
        }
        if n < *MIN_BOUND {
            return Err(UnexpectedValue(
                "ByteArrayToBigInt: byte array encodes number that's smaller than -2^255".into(),
            ));
        }
        Ok(BigInt(n))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::eval::tests::try_eval_out_wo_ctx;
    use num_bigint::{BigInt, Sign, ToBigInt};

    fn eval_node(val: Vec<u8>) -> Result<BigInt, EvalError> {
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
        assert_eq!(res, Ok(BigInt::parse_bytes(b"1", 10).unwrap()));
    }

    #[test]
    fn eval_neg1() {
        let res = eval_node(vec![0xff]);
        assert_eq!(res, Ok(BigInt::parse_bytes(b"-1", 10).unwrap()));
    }

    #[test]
    fn eval_neg1b() {
        let res = eval_node(vec![0xff, 0xff]);
        assert_eq!(res, Ok(BigInt::parse_bytes(b"-1", 10).unwrap()));
    }

    #[test]
    fn eval_256() {
        let res = eval_node(vec![1, 0]);
        assert_eq!(res, Ok(BigInt::parse_bytes(b"256", 10).unwrap()));
    }

    #[test]
    fn eval_neg256() {
        let res = eval_node(vec![0b1000_0000, 0]);
        assert_eq!(res, Ok(BigInt::parse_bytes(b"-32768", 10).unwrap()));
    }

    #[test]
    fn eval_max_bound() {
        let mut buf = vec![0xff_u8; 32];
        buf[0] = 0b0111_1111;
        match eval_node(buf) {
            Ok(n) => assert_eq!(n, *MAX_BOUND),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn eval_min_bound() {
        let mut buf = vec![0; 32];
        buf[0] = 0b1000_0000;
        match eval_node(buf) {
            Ok(n) => assert_eq!(n, *MIN_BOUND),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn eval_above_max_bound() {
        let mut buf = vec![0; 33];
        buf[1] = 0b1000_0000;
        // Check that value is just above MAX_BOUND
        assert_eq!(
            BigInt::from_bytes_be(Sign::Plus, &buf),
            MAX_BOUND.clone() + 1.to_bigint().unwrap()
        );
        let res = eval_node(buf);
        assert!(res.is_err());
    }

    #[test]
    fn eval_below_min_bound() {
        let mut buf = vec![0b1111_1111_u8; 33];
        buf[1] = 0b0111_1111;
        // Check that value is just below MIN_BOUND
        assert_eq!(
            BigInt::from_signed_bytes_be(&buf),
            MIN_BOUND.clone() - 1.to_bigint().unwrap()
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
