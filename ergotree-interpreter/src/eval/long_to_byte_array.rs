use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::constant::TryExtractInto;

impl Evaluable for LongToByteArray {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let _input_v = self.input.eval(env, ctx)?.try_extract_into::<i64>()?;
        todo!();
    }
}
