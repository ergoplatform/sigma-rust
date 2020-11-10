//! Interpreter
use crate::{
    ast::{ops::BinOp, ops::NumOp, Constant, ConstantVal, Expr},
    sigma_protocol::sigma_boolean::SigmaBoolean,
    types::SType,
};

use cost_accum::CostAccumulator;
use thiserror::Error;
use value::Value;

use self::context::Context;

pub(crate) mod context;
mod cost_accum;
mod costs;
mod value;

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
        eval(expr, env, &mut ca, ctx).and_then(|v| match v {
            Value::Boolean(b) => Ok(ReductionResult {
                sigma_prop: SigmaBoolean::TrivialProp(b),
                cost: 0,
            }),
            Value::SigmaProp(sb) => Ok(ReductionResult {
                sigma_prop: *sb,
                cost: 0,
            }),
            _ => Err(EvalError::InvalidResultType),
        })
    }
}

#[allow(unconditional_recursion)]
fn eval(
    expr: &Expr,
    env: &Env,
    ca: &mut CostAccumulator,
    ctx: &Context,
) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(b),
        }) => Ok(Value::Boolean(*b)),
        Expr::Const(Constant {
            tpe: SType::SSigmaProp,
            v: ConstantVal::SigmaProp(sp),
        }) => Ok(Value::SigmaProp(Box::new((*sp.value()).clone()))),
        Expr::Coll { .. } => todo!(),
        Expr::Tup { .. } => todo!(),
        Expr::PredefFunc(_) => todo!(),
        Expr::CollM(_) => todo!(),
        Expr::BoxM(_) => todo!(),
        Expr::CtxM(_) => todo!(),
        Expr::MethodCall { .. } => todo!(),
        Expr::BinOp(bin_op, l, r) => {
            let v_l = eval(l, env, ca, ctx)?;
            let v_r = eval(r, env, ca, ctx)?;
            ca.add_cost_of(expr);
            Ok(match bin_op {
                BinOp::Num(op) => match op {
                    NumOp::Add => v_l + v_r,
                },
            })
        }
        _ => Err(EvalError::UnexpectedExpr),
    }
}
