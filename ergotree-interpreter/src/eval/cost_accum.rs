use ergotree_ir::chain::context::CostLimitExceeded;
use thiserror::Error;

#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum CostError {
    #[error("Cost limit exceeded: {0}")]
    LimitExceeded(CostLimitExceeded),
}

impl From<CostLimitExceeded> for CostError {
    fn from(e: CostLimitExceeded) -> Self {
        CostError::LimitExceeded(e)
    }
}
