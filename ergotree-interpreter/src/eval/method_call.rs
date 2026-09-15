use alloc::vec::Vec;
use ergotree_ir::mir::method_call::MethodCall;
use ergotree_ir::mir::value::Value;

use super::smethod_eval_fn;
use super::Context;
use super::Env;
use super::EvalError;
use super::Evaluable;

impl Evaluable for MethodCall {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ectx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ectx.add_jit_cost(4)?; // MethodCall = Fixed(4)
        let ov = self.obj.eval(env, ectx)?;
        let argsv: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ectx)).collect();
        smethod_eval_fn(&self.method)?(&self.method, env, ectx, ov, argsv?)
    }
}
