#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::ast::{ops::BinOp, ops::NumOp, Expr};

use cost_accum::CostAccumulator;
use costs::{Cost, Costs};
use std::ops::Add;
use value::Value;

mod cost_accum;
mod costs;
mod value;

pub struct Env();

pub enum EvalError {}

pub trait Executor {
    fn eval(&mut self, expr: &Expr, env: &Env) -> Result<Value, EvalError>;
    fn add_cost_of(&mut self, expr: &Expr);
}

pub struct Interpreter {
    costs: Costs,
    cost_accum: CostAccumulator,
}

impl Executor for Interpreter {
    #[allow(unconditional_recursion)]
    fn eval(&mut self, expr: &Expr, env: &Env) -> Result<Value, EvalError> {
        match expr {
            Expr::Constant { tpe, v } => todo!(), //Ok(EvalResult(*v)),
            Expr::Coll { tpe, v } => todo!(),
            Expr::Tup { tpe, v } => todo!(),
            Expr::PredefFunc(_) => todo!(),
            Expr::CollM(_) => todo!(),
            Expr::BoxM(_) => todo!(),
            Expr::CtxM(_) => todo!(),
            Expr::MethodCall {
                tpe,
                obj,
                method,
                args,
            } => todo!(),
            Expr::BinOp(bin_op, l, r) => {
                let v_l = self.eval(l, env)?;
                let v_r = self.eval(r, env)?;
                self.add_cost_of(expr);
                Ok(match bin_op {
                    BinOp::Num(op) => match op {
                        NumOp::Add => v_l + v_r,
                    },
                })
            }
        }
    }

    fn add_cost_of(&mut self, expr: &Expr) {
        let cost = self.costs.cost_of(expr);
        self.cost_accum.add(cost);
    }
}
