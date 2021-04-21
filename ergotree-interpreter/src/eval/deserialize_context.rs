use ergotree_ir::mir::deserialize_context::DeserializeContext;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
// use ergotree_ir::mir::constant::TryExtractInto;
// use ergotree_ir::serialization::SigmaSerializable;

impl Evaluable for DeserializeContext {
    fn eval(&self, _env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        todo!()
    }
}
