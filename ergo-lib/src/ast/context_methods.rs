use crate::eval::cost_accum::CostAccumulator;
use crate::eval::EvalError;
use crate::eval::Evaluable;

use super::constant::Constant;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Context type instance
pub enum ContextM {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
}

impl Evaluable for ContextM {
    fn eval(
        &self,
        _env: &crate::eval::Env,
        _ca: &mut CostAccumulator,
        ctx: &crate::eval::context::Context,
    ) -> Result<Constant, EvalError> {
        match self {
            ContextM::Height => Ok(ctx.height.clone()),
            _ => Err(EvalError::UnexpectedExpr),
        }
    }
}
