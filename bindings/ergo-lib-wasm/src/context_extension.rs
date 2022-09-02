//! ProverResult

use crate::ast::Constant;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// User-defined variables to be put into context
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ContextExtension(
    ergo_lib::ergotree_interpreter::sigma_protocol::prover::ContextExtension,
);

#[wasm_bindgen]
impl ContextExtension {
    /// Create new ContextExtension instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(ergo_lib::ergotree_interpreter::sigma_protocol::prover::ContextExtension::empty())
    }

    /// Set the supplied pair in the ContextExtension
    pub fn set_pair(&mut self, id: u8, value: &Constant) {
        self.0.values.insert(id, value.clone().into());
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        let wrapped: ergo_lib::ergotree_interpreter::sigma_protocol::prover::ContextExtension =
            self.0.clone();
        wrapped.values.len()
    }
    /// get from map or fail if key is missing
    pub fn get(&self, key: u8) -> Result<Constant, JsValue> {
        let wrapped: ergo_lib::ergotree_interpreter::sigma_protocol::prover::ContextExtension =
            self.0.clone();
        Ok(wrapped
            .values
            .get(&key)
            .ok_or_else::<JsValue, _>(|| "err".into())?
            .clone()
            .into())
    }

    /// Returns all keys in the map
    pub fn keys(&self) -> Vec<u8> {
        let wrapped: ergo_lib::ergotree_interpreter::sigma_protocol::prover::ContextExtension =
            self.0.clone();
        wrapped.values.keys().cloned().collect()
    }

    /// Returns serialized bytes or fails with error if ContextExtension cannot be serialized
    pub fn sigma_serialize_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0
            .sigma_serialize_bytes()
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
    }
}
