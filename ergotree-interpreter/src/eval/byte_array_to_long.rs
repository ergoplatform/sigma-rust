use ergotree_ir::mir::byte_array_to_long::ByteArrayToLong;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::constant::TryExtractInto;

impl Evaluable for ByteArrayToLong {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let _input_v = self.input.eval(env, ctx)?.try_extract_into::<Vec<i8>>()?;
        todo!();
    }
}
