//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;
use ergo_lib::chain::ergo_state_context::Headers;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::block_header::BlockHeaders;
use crate::header::PreHeader;
use crate::parameters::Parameters;

/// Blockchain state (last headers, etc.)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoStateContext(pub(crate) chain::ergo_state_context::ErgoStateContext);

#[wasm_bindgen]
impl ErgoStateContext {
    /// Create new context from pre-header
    #[wasm_bindgen(constructor)]
    pub fn new(
        pre_header: PreHeader,
        headers: BlockHeaders,
        parameters: Parameters,
    ) -> Result<ErgoStateContext, JsValue> {
        let headers = Headers::try_from(headers)?;
        Ok(chain::ergo_state_context::ErgoStateContext::new(
            pre_header.into(),
            headers,
            parameters.into(),
        )
        .into())
    }
}
