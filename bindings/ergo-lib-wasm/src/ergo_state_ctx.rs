//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// TBD
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoStateContext(chain::ergo_state_context::ErgoStateContext);

#[wasm_bindgen]
impl ErgoStateContext {
    /// empty (dummy) context (for signing P2PK tx only)
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext(chain::ergo_state_context::ErgoStateContext::dummy())
    }
}

// TODO: add PreHeader, and build it from BlockHeader
// TODO: add BlockHeader and parse them from JSON (REST API lastHeaders)
