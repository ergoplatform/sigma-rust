//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::header::PreHeader;

/// Blockchain state (last headers, etc.)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoStateContext(chain::ergo_state_context::ErgoStateContext);

#[wasm_bindgen]
impl ErgoStateContext {
    /// Create new context from pre-header
    #[wasm_bindgen(constructor)]
    pub fn new(pre_header: PreHeader) -> Self {
        chain::ergo_state_context::ErgoStateContext {
            pre_header: pre_header.into(),
        }
        .into()
    }

    /// empty (dummy) context (for signing P2PK tx only)
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext(chain::ergo_state_context::ErgoStateContext::dummy())
    }
}
