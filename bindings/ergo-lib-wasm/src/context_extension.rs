//! ProverResult

use crate::ast::Constant;
use ergo_lib::sigma_protocol;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// Proof of correctness of tx spending
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, From, Into)]
pub struct ContextExtension(sigma_protocol::prover::ContextExtension);

#[wasm_bindgen]
impl ContextExtension {
    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.values.len()
    }
    /// get from map or fail if key is missing
    pub fn get(&self, key: u8) -> Constant {
        self.0.values.get(&key).unwrap().clone().into()
    }

    /// Returns all keys in the map
    pub fn keys(&self) -> Vec<u8> {
        self.0.values.keys().cloned().collect()
    }
}
