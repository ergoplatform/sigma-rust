use crate::ast::global_vars::GlobalVars;
use crate::ast::value::Value;

use super::cost_accum::CostAccumulator;
use super::EvalError;
use super::Evaluable;

impl Evaluable for GlobalVars {
    fn eval(
        &self,
        _env: &crate::eval::Env,
        _ca: &mut CostAccumulator,
        ctx: &crate::eval::context::Context,
    ) -> Result<Value, EvalError> {
        match self {
            GlobalVars::Height => Ok(ctx.height.clone().into()),
            GlobalVars::SelfBox => Ok(ctx.self_box.clone().into()),
            GlobalVars::Outputs => Ok(ctx.outputs.clone().into()),
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
    fn eval_height() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<i32>(&GlobalVars::Height.into(), &ctx),
            ctx.height
        );
    }

    #[test]
    fn eval_self_box() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<ErgoBox>(&GlobalVars::SelfBox.into(), &ctx),
            ctx.self_box
        );
    }

    #[test]
    fn eval_outputs() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<ErgoBox>>(&GlobalVars::Outputs.into(), &ctx),
            ctx.outputs
        );
    }
}
