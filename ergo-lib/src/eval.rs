//! Interpreter
use crate::{
    ast::{Constant, ConstantVal, Expr},
    sigma_protocol::sigma_boolean::SigmaBoolean,
    types::SType,
};

use cost_accum::CostAccumulator;
use thiserror::Error;

use self::context::Context;

pub(crate) mod context;
pub(crate) mod cost_accum;
mod costs;

/// Environment vars for script interpreter
pub struct Env();

impl Env {
    /// Empty environment
    pub fn empty() -> Env {
        Env()
    }
}

/// Interpreter errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum EvalError {
    /// Only boolean or SigmaBoolean is a valid result expr type
    #[error("Only boolean or SigmaBoolean is a valid result expr type")]
    InvalidResultType,
    /// Unsupported Expr encountered during the evaluation
    #[error("Unsupported Expr encountered during the evaluation")]
    // TODO: store unexpected expr
    UnexpectedExpr,
}

/// Result of ErgoTree reduction procedure (see `reduce_to_crypto`).
pub struct ReductionResult {
    /// value of SigmaProp type which represents a statement verifiable via sigma protocol.
    pub sigma_prop: SigmaBoolean,
    /// estimated cost of contract execution
    pub cost: u64,
}

/// Interpreter
pub trait Evaluator {
    /// This method is used in both prover and verifier to compute SigmaBoolean value.
    fn reduce_to_crypto(
        &self,
        expr: &Expr,
        env: &Env,
        ctx: &Context,
    ) -> Result<ReductionResult, EvalError> {
        let mut ca = CostAccumulator::new(0, None);
        eval(expr, env, &mut ca, ctx).and_then(|v| -> Result<ReductionResult, EvalError> {
            match v {
                Constant {
                    tpe: SType::SBoolean,
                    v: ConstantVal::Boolean(b),
                } => Ok(ReductionResult {
                    sigma_prop: SigmaBoolean::TrivialProp(b),
                    cost: 0,
                }),
                Constant {
                    tpe: SType::SSigmaProp,
                    v: ConstantVal::SigmaProp(sp),
                } => Ok(ReductionResult {
                    sigma_prop: sp.value().clone(),
                    cost: 0,
                }),
                _ => Err(EvalError::InvalidResultType),
            }
        })
    }
}

/// Implemented by every node that can be evaluated
pub trait Evaluable {
    /// Evaluation routine to be implement by each node
    fn eval(
        &self,
        env: &Env,
        ca: &mut CostAccumulator,
        ctx: &Context,
    ) -> Result<Constant, EvalError>;
}

#[allow(unconditional_recursion)]
fn eval(
    expr: &Expr,
    env: &Env,
    ca: &mut CostAccumulator,
    ctx: &Context,
) -> Result<Constant, EvalError> {
    match expr {
        Expr::Const(c) => Ok(c.clone()),
        Expr::Coll { .. } => todo!(),
        Expr::Tup { .. } => todo!(),
        Expr::PredefFunc(_) => todo!(),
        Expr::CollM(_) => todo!(),
        Expr::BoxM(_) => todo!(),
        Expr::CtxM(v) => v.eval(env, ca, ctx),
        Expr::MethodCall { .. } => todo!(),
        Expr::BinOp(_bin_op, l, r) => {
            let _v_l = eval(l, env, ca, ctx)?;
            let _v_r = eval(r, env, ca, ctx)?;
            ca.add_cost_of(expr);
            todo!()
            // Ok(match bin_op {
            //     BinOp::Num(op) => match op {
            //         NumOp::Add => v_l + v_r,
            //     },
            // })
        }
        _ => Err(EvalError::UnexpectedExpr),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::ContextMethods;
    use crate::ast::TryExtractFrom;

    use super::*;

    #[test]
    fn height() {
        let expr = Expr::CtxM(ContextMethods::Height);
        let mut ca = CostAccumulator::new(0, None);
        let res = eval(&expr, &Env::empty(), &mut ca, &Context::dummy()).unwrap();
        assert_eq!(i32::try_extract_from(res).unwrap(), 0);
    }
}
