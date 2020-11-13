use super::costs::{Cost, Costs};
use crate::ast::expr::Expr;

pub struct CostAccumulator {
    costs: Costs,
}

impl CostAccumulator {
    pub fn new(_initial_cost: u64, _cost_limit: Option<u64>) -> CostAccumulator {
        CostAccumulator {
            costs: Costs::DEFAULT,
        }
    }

    pub fn add_cost_of(&mut self, expr: &Expr) {
        let cost = self.costs.cost_of(expr);
        self.add(cost);
    }

    pub fn add(&self, _: Cost) {}
}
