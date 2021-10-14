//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::header::PreHeader;

/// Fixed number of last block headers in descending order (first header is the newest one)
#[wasm_bindgen]
#[derive(From, Into)]
pub struct ErgoStateContextHeaders(chain::ergo_state_context::ErgoStateContextHeaders);

/// Blockchain state (last headers, etc.)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoStateContext(chain::ergo_state_context::ErgoStateContext);

#[wasm_bindgen]
impl ErgoStateContext {
    /// Create new context from pre-header
    #[wasm_bindgen(constructor)]
    pub fn new(pre_header: PreHeader, headers: ErgoStateContextHeaders) -> Self {
        let ergo_state_context =
            chain::ergo_state_context::ErgoStateContext::new(pre_header.into(), headers.into());
        ergo_state_context.into()
    }
}
