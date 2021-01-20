use crate::ast::property_call::PropertyCall;
use crate::ast::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for PropertyCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = self.obj.eval(env, ectx)?;
        self.method.eval_fn()(ov, vec![])
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
        let pc: Expr = PropertyCall {
            obj: Box::new(Expr::Context),
            method: scontext::DATA_INPUTS_PROPERTY.clone(),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<ErgoBox>>(&pc, ctx.clone()), ctx.data_inputs);
    }
}
