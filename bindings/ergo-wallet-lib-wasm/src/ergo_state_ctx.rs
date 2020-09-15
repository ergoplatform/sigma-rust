use sigma_tree::chain;
use wasm_bindgen::prelude::*;

/// TBD
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext(chain::ergo_state_context::ErgoStateContext);

#[wasm_bindgen]
impl ErgoStateContext {
    /// empty (dummy) context (for signing P2PK tx only)
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext(chain::ergo_state_context::ErgoStateContext::dummy())
    }
}

impl Into<chain::ergo_state_context::ErgoStateContext> for ErgoStateContext {
    fn into(self) -> chain::ergo_state_context::ErgoStateContext {
        self.0
    }
}
