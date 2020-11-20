use crate::eval::cost_accum::CostAccumulator;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;

use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Context type instance
pub enum ContextM {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
    /// ErgoBox instance, which script is being evaluated
    SelfBox,
}

impl ContextM {
    pub fn op_code(&self) -> OpCode {
        match self {
            ContextM::SelfBox => OpCode::SELF_BOX,
            _ => todo!(),
        }
    }
}

impl Evaluable for ContextM {
    fn eval(
        &self,
        _env: &crate::eval::Env,
        _ca: &mut CostAccumulator,
        ctx: &crate::eval::context::Context,
    ) -> Result<Value, EvalError> {
        match self {
            ContextM::Height => Ok(ctx.height.clone().into()),
            // TODO: test
            ContextM::SelfBox => Ok(ctx.self_box.clone().into()),
            ContextM::Outputs => Ok(ctx.outputs.clone().into()),
            _ => Err(EvalError::UnexpectedExpr),
        }
    }
}
