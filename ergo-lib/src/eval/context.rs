use crate::ast::Constant;
use crate::chain::ergo_state_context::ErgoStateContext;
use crate::wallet::signing::TransactionContext;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
    pub height: Constant,
}

impl Context {
    #[cfg(test)]
    pub fn dummy() -> Self {
        Context {
            height: 0i32.into(),
        }
    }

    pub fn new(
        _state_ctx: &ErgoStateContext,
        _tx_ctx: &TransactionContext,
        _self_index: usize,
    ) -> Self {
        // TODO: implement
        Context {
            height: 0i32.into(),
        }
    }
}
