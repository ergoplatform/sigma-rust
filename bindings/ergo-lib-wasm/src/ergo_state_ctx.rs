//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::header::PreHeader;
use crate::block_header::BlockHeaders;

/// Blockchain state (last headers, etc.)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoStateContext(chain::ergo_state_context::ErgoStateContext);

#[wasm_bindgen]
impl ErgoStateContext {
    /// Create new context from pre-header
    #[wasm_bindgen(constructor)]
    pub fn new(pre_header: PreHeader, headers: BlockHeaders) -> Self {
        let ergo_state_context =
            chain::ergo_state_context::ErgoStateContext::new(pre_header.into(), headers.into());
        ergo_state_context.into()
    }
}
