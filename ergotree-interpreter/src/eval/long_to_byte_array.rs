use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::constant::TryExtractInto;

impl Evaluable for LongToByteArray {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(17)?; // LongToByteArray = Fixed(17)
        let mut val = self.input.eval(env, ctx)?.try_extract_into::<i64>()?;
        let mut buf = vec![42_i8; 8];
        for i in (0..8).rev() {
            buf[i] = (val & 0xFF) as i8;
            val >>= 8;
        }
        Ok(buf.into())
    }
}

#[cfg(test)]
mod tests {

    use alloc::{boxed::Box, vec::Vec};

    use super::*;
    use crate::eval::test_util::eval_out_wo_ctx;

    fn eval_node(val: i64) -> Vec<i8> {
        let expr = LongToByteArray {
            input: Box::new(val.into()),
        }
        .into();
        eval_out_wo_ctx(&expr)
    }

    #[test]
    fn eval_1() {
        let res = eval_node(1);
        assert_eq!(res, vec![0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn eval_neg1() {
        let res = eval_node(-1);
        assert_eq!(res, vec![-1; 8]);
    }

    #[test]
    fn eval_big() {
        let res = eval_node(0x11_12_13_14_15_16_17_18_i64);
        assert_eq!(res, vec![0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18]);
    }
}
