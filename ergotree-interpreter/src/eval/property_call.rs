use ergotree_ir::mir::property_call::PropertyCall;
use ergotree_ir::mir::value::Value;

use super::smethod_eval_fn;
use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for PropertyCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = self.obj.eval(env, ectx)?;
        smethod_eval_fn(&self.method)?(env, ectx, ov, vec![])
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::types::scontext;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn eval_context_data_inputs() {
        let pc: Expr = PropertyCall::new(Expr::Context, scontext::DATA_INPUTS_PROPERTY.clone())
            .unwrap()
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .data_inputs
            .clone()
            .map_or(vec![], |d| d.as_vec().clone());
        assert_eq!(eval_out::<Vec<Rc<ErgoBox>>>(&pc, ctx), expected,);
    }
}
