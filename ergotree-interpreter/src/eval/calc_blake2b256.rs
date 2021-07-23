use ergotree_ir::mir::calc_blake2b256::CalcBlake2b256;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::util::AsVecU8;
use sigma_util::hash::blake2b256_hash;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for CalcBlake2b256 {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v.clone() {
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(coll_byte))) => {
                let expected_hash: Vec<u8> =
                    blake2b256_hash(coll_byte.as_vec_u8().as_slice()).to_vec();
                Ok(expected_hash.into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected CalcBlake2b256 input to be byte array, got: {0:?}",
                input_v
            ))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    proptest! {

        #[test]
        fn eval(byte_array in any::<Vec<u8>>()) {
            let expected_hash = blake2b256_hash(byte_array.as_slice()).to_vec();
            let expr: Expr = CalcBlake2b256 {
                input: Box::new(Expr::Const(byte_array.into())),
            }
            .into();
            let ctx = Rc::new(force_any_val::<Context>());
            assert_eq!(eval_out::<Vec<i8>>(&expr, ctx).as_vec_u8(), expected_hash);
        }

    }
}
