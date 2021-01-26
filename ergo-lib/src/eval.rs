//! Interpreter
use std::rc::Rc;

use crate::ast::constant::TryExtractFromError;
use crate::ast::expr::Expr;
use crate::ast::value::Value;
use crate::chain::ergo_box::RegisterIdOutOfBounds;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;

use cost_accum::CostAccumulator;
use thiserror::Error;

use self::context::Context;
use self::cost_accum::CostError;
use self::env::Env;

pub(crate) mod context;
pub(crate) mod cost_accum;
pub(crate) mod costs;
pub(crate) mod env;
pub(crate) mod expr;
pub(crate) mod global_vars;
pub(crate) mod method_call;
pub(crate) mod property_call;

/// Interpreter errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum EvalError {
    /// Only boolean or SigmaBoolean is a valid result expr type
    #[error("Only boolean or SigmaBoolean is a valid result expr type")]
    InvalidResultType,
    /// Unexpected Expr encountered during the evaluation
    #[error("Unexpected Expr: {0:?}")]
    UnexpectedExpr(String),
    /// Error on cost calculation
    #[error("Error on cost calculation: {0:?}")]
    CostError(#[from] CostError),
    /// Unexpected value type
    #[error("Unexpected value type: {0:?}")]
    TryExtractFrom(#[from] TryExtractFromError),
    /// Not found (missing value, argument, etc.)
    #[error("Not found: {0}")]
    NotFound(String),
    /// Register id out of bounds
    #[error("{0:?}")]
    RegisterIdOutOfBounds(#[from] RegisterIdOutOfBounds),
    /// Unexpected value
    #[error("Unexpected value: {0:?}")]
    UnexpectedValue(String),
    /// Arithmetic exception error
    #[error("Arithmetic exception: {0:?}")]
    ArithmeticException(String),
}

/// Result of expression reduction procedure (see `reduce_to_crypto`).
pub struct ReductionResult {
    /// value of SigmaProp type which represents a statement verifiable via sigma protocol.
    pub sigma_prop: SigmaBoolean,
    /// estimated cost of expression evaluation
    pub cost: u64,
}

/// Interpreter
pub trait Evaluator {
    /// Evaluate the given expression by reducing it to SigmaBoolean value.
    fn reduce_to_crypto(
        &self,
        expr: &Expr,
        env: &Env,
        ctx: Rc<Context>,
    ) -> Result<ReductionResult, EvalError> {
        let cost_accum = CostAccumulator::new(0, None);
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(env, &mut ectx)
            .and_then(|v| -> Result<ReductionResult, EvalError> {
                match v {
                    Value::Boolean(b) => Ok(ReductionResult {
                        sigma_prop: SigmaBoolean::TrivialProp(b),
                        cost: 0,
                    }),
                    Value::SigmaProp(sp) => Ok(ReductionResult {
                        sigma_prop: sp.value().clone(),
                        cost: 0,
                    }),
                    _ => Err(EvalError::InvalidResultType),
                }
            })
    }
}

pub struct EvalContext {
    pub ctx: Rc<Context>,
    pub cost_accum: CostAccumulator,
}

impl EvalContext {
    pub fn new(ctx: Rc<Context>, cost_accum: CostAccumulator) -> Self {
        EvalContext { ctx, cost_accum }
    }
}

/// Expression evaluation.
/// Should be implemented by every node that can be evaluated.
pub trait Evaluable {
    /// Evaluation routine to be implement by each node
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError>;
}

#[cfg(test)]
pub mod tests {

    use crate::ast::constant::TryExtractFrom;
    use crate::ast::constant::TryExtractInto;

    use super::env::Env;
    use super::*;

    pub fn eval_out_wo_ctx<T: TryExtractFrom<Value>>(expr: &Expr) -> T {
        let cost_accum = CostAccumulator::new(0, None);
        let ctx = Rc::new(Context::dummy());
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(&Env::empty(), &mut ectx)
            .unwrap()
            .try_extract_into::<T>()
            .unwrap()
    }

    pub fn eval_out<T: TryExtractFrom<Value>>(expr: &Expr, ctx: Rc<Context>) -> T {
        let cost_accum = CostAccumulator::new(0, None);
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(&Env::empty(), &mut ectx)
            .unwrap()
            .try_extract_into::<T>()
            .unwrap()
    }

    pub fn try_eval_out<T: TryExtractFrom<Value>>(
        expr: &Expr,
        ctx: Rc<Context>,
    ) -> Result<T, EvalError> {
        let cost_accum = CostAccumulator::new(0, None);
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(&Env::empty(), &mut ectx)
            .and_then(|v| v.try_extract_into::<T>().map_err(EvalError::TryExtractFrom))
    }
}
