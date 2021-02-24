use crate::mir::property_call::PropertyCall;
use crate::mir::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for PropertyCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = self.obj.eval(env, ectx)?;
        self.method.eval_fn()(ectx.ctx.clone(), ov, vec![])
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::ir_ergo_box::IrBoxId;
    use crate::mir::expr::Expr;
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
        assert_eq!(eval_out::<Vec<IrBoxId>>(&pc, ctx.clone()), ctx.data_inputs);
    }
}
