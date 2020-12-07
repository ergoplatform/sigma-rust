use crate::ast::method_call::MethodCall;
use crate::ast::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for MethodCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = (*self.obj).eval(env, ectx)?;
        let argsv: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ectx)).collect();
        // TODO: check evaluated object and arg values types with declared (in SMethod)
        self.method.eval_fn()(ov, argsv?)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::expr::Expr;
    use crate::chain::ergo_box::ErgoBox;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;
    use crate::types::scontext;

    use super::*;

    #[test]
    fn eval_context_data_inputs() {
        let mc = MethodCall {
            tpe: scontext::DATA_INPUTS_METHOD.tpe().clone(),
            obj: Box::new(Expr::Context),
            method: scontext::DATA_INPUTS_METHOD.clone(),
            args: vec![],
        };
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<ErgoBox>>(&mc.into(), ctx.clone()),
            ctx.data_inputs
        );
    }
}
