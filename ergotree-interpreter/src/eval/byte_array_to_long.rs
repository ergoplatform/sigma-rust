use ergotree_ir::mir::byte_array_to_long::ByteArrayToLong;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::EvalError::UnexpectedValue;
use crate::eval::Evaluable;
use ergotree_ir::mir::constant::TryExtractInto;

impl Evaluable for ByteArrayToLong {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input = self.input.eval(env, ctx)?.try_extract_into::<Vec<i8>>()?;
        if input.len() < 8 {
            return Err(UnexpectedValue(
                "byteArrayToLong: array must contain at least 8 elements".into(),
            ));
        }
        Ok(((input[0] as i64) << 56
            | (input[1] as i64) << 48
            | (input[2] as i64) << 40
            | (input[3] as i64) << 32
            | (input[4] as i64) << 24
            | (input[5] as i64) << 16
            | (input[6] as i64) << 8
            | (input[7] as i64))
            .into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::eval::tests::try_eval_out_wo_ctx;

    fn eval_node(val: Vec<i8>) -> Result<i64, EvalError> {
        let expr = ByteArrayToLong {
            input: Box::new(val.into()),
        }
        .into();
        try_eval_out_wo_ctx(&expr)
    }

    #[test]
    fn eval_1() {
        let res = eval_node(vec![0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(res, Ok(1));
    }

    // Bytes after first 8 are ignored
    #[test]
    fn eval_skip_tail() {
        let res = eval_node(vec![0, 0, 0, 0, 0, 0, 0, 1, 0x42, 0x42]);
        assert_eq!(res, Ok(1));
    }

    #[test]
    fn eval_neg1() {
        let res = eval_node(vec![-1; 8]);
        assert_eq!(res, Ok(-1));
    }

    #[test]
    fn fails_for_short() {
        let res = eval_node(vec![0; 7]);
        assert!(res.is_err());
    }
}
