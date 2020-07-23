use super::costs::{Cost, Costs};
use crate::ast::Expr;

pub struct CostAccumulator {
    costs: Costs,
}

impl CostAccumulator {
    pub fn new(initial_cost: u64, cost_limit: Option<u64>) -> CostAccumulator {
        todo!()
    }

    pub fn add_cost_of(&mut self, expr: &Expr) {
        let cost = self.costs.cost_of(expr);
        self.add(cost);
    }

    pub fn add(&self, _: Cost) {
        todo!();
    }
}
