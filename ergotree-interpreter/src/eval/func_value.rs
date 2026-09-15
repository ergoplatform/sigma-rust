use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::value::Lambda;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for FuncValue {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        Ok(Value::Lambda(Lambda {
            args: self.args().to_vec(),
            body: self.body().clone().into(),
            // The JVM's `FuncValue.eval` returns a closure over the defining
            // env — capture it so a lambda that escapes its creation site
            // (returned from another lambda, bound to a `val`) still sees
            // these bindings when applied.
            captured: env.bindings(),
        }))
    }
}
