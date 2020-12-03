use crate::ast::context_methods::ContextM;
use crate::ast::value::Value;

use super::cost_accum::CostAccumulator;
use super::EvalError;
use super::Evaluable;

impl Evaluable for ContextM {
    fn eval(
        &self,
        _env: &crate::eval::Env,
        _ca: &mut CostAccumulator,
        ctx: &crate::eval::context::Context,
    ) -> Result<Value, EvalError> {
        match self {
            ContextM::DataInputs => Ok(ctx.data_inputs.clone().into()),
            _ => Err(EvalError::UnexpectedExpr),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::ergo_box::ErgoBox;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_data_inputs() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<ErgoBox>>(&ContextM::DataInputs.into(), &ctx),
            ctx.data_inputs
        );
    }
}
