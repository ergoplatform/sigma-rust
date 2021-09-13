//! ProverResult

use wasm_bindgen::prelude::*;

use crate::{context_extension::ContextExtension, error_conversion::to_js};
extern crate derive_more;
use derive_more::{From, Into};

/// Proof of correctness of tx spending
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, From, Into)]
pub struct ProverResult(ergo_lib::chain::transaction::input::prover_result::ProverResult);

#[wasm_bindgen]
impl ProverResult {
    /// Get proof
    pub fn proof(&self) -> Vec<u8> {
        self.0.proof.clone().into()
    }

    /// Get extension
    pub fn extension(&self) -> ContextExtension {
        self.0.extension.clone().into()
    }

    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string_pretty(&self.0.clone())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}
