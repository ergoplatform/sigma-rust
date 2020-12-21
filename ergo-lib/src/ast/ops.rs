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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::constant::Constant;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn num_eq_op() {
        let left: Constant = 1i64.into();
        let right: Constant = 1i64.into();
        let eq_op: Expr = Box::new(BinOp {
            kind: BinOpKind::Logic(LogicOp::Eq),
            left: Box::new(left).into(),
            right: Box::new(right).into(),
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert!(eval_out::<bool>(&eq_op, ctx));
    }

    #[test]
    fn num_eq_op_fail() {
        let left: Constant = 1i64.into();
        let right: Constant = 2i64.into();
        let eq_op: Expr = Box::new(BinOp {
            kind: BinOpKind::Logic(LogicOp::Eq),
            left: Box::new(left).into(),
            right: Box::new(right).into(),
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert!(!eval_out::<bool>(&eq_op, ctx));
    }

    #[test]
    fn option_eq_op() {
        let left: Constant = Some(1i64).into();
        let right: Constant = Some(1i64).into();
        let eq_op: Expr = Box::new(BinOp {
            kind: BinOpKind::Logic(LogicOp::Eq),
            left: Box::new(left).into(),
            right: Box::new(right).into(),
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert!(eval_out::<bool>(&eq_op, ctx));
    }

    #[test]
    fn option_eq_op_fail() {
        let left: Constant = Some(1i64).into();
        let right: Constant = Some(2i64).into();
        let eq_op: Expr = Box::new(BinOp {
            kind: BinOpKind::Logic(LogicOp::Eq),
            left: Box::new(left).into(),
            right: Box::new(right).into(),
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert!(!eval_out::<bool>(&eq_op, ctx));
    }

    // TODO: add Some(_) != None test for Option
}
