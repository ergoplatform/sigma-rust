//! Operators in ErgoTree

use crate::eval::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

use super::expr::Expr;
use super::value::Value;
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// Operations for numerical types
pub enum NumOp {
    /// Addition
    Add,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LogicOp {
    Eq,
    Neq,
    GE,
    GT,
    LE,
    LT,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// Binary operations
pub enum BinOpKind {
    /// Binary operations for numerical types
    Num(NumOp),
    Logic(LogicOp),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub left: Expr,
    pub right: Expr,
}

impl Evaluable for BinOp {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let lv = self.left.eval(env, ctx)?;
        let rv = self.right.eval(env, ctx)?;
        // TODO: costing
        // ctx.cost_accum.add(cost)
        match self.kind {
            BinOpKind::Num(_) => todo!(),
            BinOpKind::Logic(op) => match op {
                LogicOp::Eq => Ok(Value::Boolean(lv == rv)),
                _ => todo!(),
            },
        }
    }
}
