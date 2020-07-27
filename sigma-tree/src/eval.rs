use crate::{
    ast::{ops::BinOp, ops::NumOp, Constant, ConstantVal, Expr},
    sigma_protocol::SigmaBoolean,
    types::SType,
};

use cost_accum::CostAccumulator;
use value::Value;

mod cost_accum;
mod costs;
mod value;

pub struct Env();

impl Env {
    pub fn empty() -> Env {
        Env()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum EvalError {
    InvalidResultType,
    // TODO: store unexpected expr
    UnexpectedExpr,
}

pub trait Evaluator {
    // TODO: add the cost to the returned result
    fn reduce_to_crypto(&self, expr: &Expr, env: &Env) -> Result<SigmaBoolean, EvalError> {
        let mut ca = CostAccumulator::new(0, None);
        eval(expr, env, &mut ca).and_then(|v| match v {
            Value::Boolean(b) => Ok(SigmaBoolean::TrivialProp(b)),
            Value::SigmaProp(sb) => Ok(*sb),
            _ => Err(EvalError::InvalidResultType),
        })
    }
}

#[allow(unconditional_recursion)]
fn eval(expr: &Expr, env: &Env, ca: &mut CostAccumulator) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(b),
        }) => Ok(Value::Boolean(*b)), //Ok(EvalResult(*v)),
        Expr::Coll { .. } => todo!(),
        Expr::Tup { .. } => todo!(),
        Expr::PredefFunc(_) => todo!(),
        Expr::CollM(_) => todo!(),
        Expr::BoxM(_) => todo!(),
        Expr::CtxM(_) => todo!(),
        Expr::MethodCall { .. } => todo!(),
        Expr::BinOp(bin_op, l, r) => {
            let v_l = eval(l, env, ca)?;
            let v_r = eval(r, env, ca)?;
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

