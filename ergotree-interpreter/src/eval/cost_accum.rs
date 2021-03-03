use super::costs::{Cost, Costs};
use ergotree_ir::mir::expr::Expr;
use thiserror::Error;

#[derive(Debug)]
pub struct CostAccumulator {
    costs: Costs,
    accum: u64,
    limit: Option<u64>,
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum CostError {
    #[error("Limit ({0}) exceeded")]
    LimitExceeded(u64),
}

impl CostAccumulator {
    pub fn new(initial_cost: u64, cost_limit: Option<u64>) -> CostAccumulator {
        CostAccumulator {
            costs: Costs::DEFAULT,
            accum: initial_cost,
            limit: cost_limit,
        }
    }

    pub fn add_cost_of(&mut self, expr: &Expr) -> Result<(), CostError> {
        let cost = self.costs.cost_of(expr);
        self.add(cost)
    }

    pub fn add(&mut self, cost: Cost) -> Result<(), CostError> {
        self.accum += u32::from(cost) as u64;
        if let Some(limit) = self.limit {
            if self.accum > limit {
                return Err(CostError::LimitExceeded(limit));
            }
        }
        Ok(())
    }
}
