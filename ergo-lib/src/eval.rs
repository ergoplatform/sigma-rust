//! Interpreter
use crate::ast::expr::Expr;
use crate::ast::value::Value;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;

use cost_accum::CostAccumulator;
use thiserror::Error;

use self::context::Context;

mod costs;

pub(crate) mod context;
pub(crate) mod cost_accum;
pub(crate) mod global_vars;

/// Environment for the interpreter
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
        ctx: &Context,
    ) -> Result<ReductionResult, EvalError> {
        let mut ca = CostAccumulator::new(0, None);
        eval(expr, env, &mut ca, ctx).and_then(|v| -> Result<ReductionResult, EvalError> {
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

/// Expression evaluation.
/// Should be implemented by every node that can be evaluated.
pub trait Evaluable {
    /// Evaluation routine to be implement by each node
    fn eval(&self, env: &Env, ca: &mut CostAccumulator, ctx: &Context) -> Result<Value, EvalError>;
}

#[allow(unconditional_recursion)]
fn eval(
    expr: &Expr,
    env: &Env,
    ca: &mut CostAccumulator,
    ctx: &Context,
) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(c) => Ok(c.v.clone()),
        Expr::Coll { .. } => todo!(),
        Expr::Tup { .. } => todo!(),
        Expr::PredefFunc(_) => todo!(),
        Expr::CollM(_) => todo!(),
        Expr::BoxM(_) => todo!(),
        Expr::GlobalVars(v) => v.eval(env, ca, ctx),
        Expr::MethodCall(v) => v.eval(env, ca, ctx),
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
pub mod tests {

    use crate::ast::constant::TryExtractFrom;
    use crate::ast::global_vars::GlobalVars;

    use super::*;

    pub fn eval_out<T: TryExtractFrom<Value>>(expr: &Expr, ctx: &Context) -> T {
        use crate::ast::constant::TryExtractInto;
        let mut ca = CostAccumulator::new(0, None);
        eval(expr, &Env::empty(), &mut ca, ctx)
            .unwrap()
            .try_extract_into::<T>()
            .unwrap()
    }

    #[test]
    fn height() {
        let expr = Expr::GlobalVars(GlobalVars::Height);
        let mut ca = CostAccumulator::new(0, None);
        let res = eval(&expr, &Env::empty(), &mut ca, &Context::dummy()).unwrap();
        assert_eq!(i32::try_extract_from(res).unwrap(), 0);
    }
}
