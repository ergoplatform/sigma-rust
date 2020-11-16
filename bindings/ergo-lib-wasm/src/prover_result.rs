//! ProverResult

use wasm_bindgen::prelude::*;
use ergo_lib::sigma_protocol;
use ergo_lib::serialization::SigmaSerializable;

use crate::context_extension::ContextExtension;
extern crate derive_more;
use derive_more::{From, Into};

/// Proof of correctness of tx spending
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, From, Into)]
pub struct ProverResult(sigma_protocol::prover::ProverResult);

#[wasm_bindgen]
impl ProverResult {
    /// Get proof
    pub fn proof(&self) -> Vec<u8> {
        self.0.proof.sigma_serialize_bytes()
    }

    /// Get extension
    pub fn extension(&self) -> ContextExtension {
        self.0.extension.clone().into()
    }

    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0.clone()).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}
