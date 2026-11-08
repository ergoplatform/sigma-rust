use ergotree_ir::mir::property_call::PropertyCall;
use ergotree_ir::mir::value::Value;

use super::smethod_eval_fn;
use super::Context;
use super::Env;
use super::EvalError;
use super::Evaluable;

impl Evaluable for PropertyCall {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ectx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ectx.add_jit_cost(4)?; // PropertyCall = Fixed(4)
        let ov = self.obj.eval(env, ectx)?;
        smethod_eval_fn(&self.method)?(&self.method, env, ectx, ov, vec![])
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::reference::Ref;
    use ergotree_ir::types::scontext;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval_context_data_inputs() {
        let pc: Expr = PropertyCall::new(Expr::Context, scontext::DATA_INPUTS_PROPERTY.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        let expected = ctx
            .data_inputs
            .clone()
            .map_or(vec![], |d| d.as_vec().clone());
        eval_out::<Vec<Ref<'_, ErgoBox>>>(&pc, &ctx)
            .into_iter()
            .zip(expected)
            .for_each(|(a, b)| assert_eq!(&*a, b));
    }
}
