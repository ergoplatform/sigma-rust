//! WASM bindings for sigma-tree

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]

use sigma_tree::chain;

mod utils;

// use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// TODO: wrap sigma-tree type
#[wasm_bindgen]
pub struct Address(String);

#[wasm_bindgen]
impl Address {
    /// Decode(base58) address
    pub fn from_str(str: String) -> Address {
        Address(str)
    }
}

/// TODO: wrap sigma-tree type
#[wasm_bindgen]
pub struct PrivateKey(String);

#[wasm_bindgen]
impl PrivateKey {
    /// Decode from string
    pub fn from_str(str: String) -> PrivateKey {
        PrivateKey(str)
    }
}

/// Transaction inputs
#[wasm_bindgen]
pub struct TxInputs(Vec<chain::ErgoBoxCandidate>);

#[wasm_bindgen]
impl TxInputs {
    /// parse ErgoBoxCandidate from json
    pub fn from_boxes(_boxes: Box<[JsValue]>) -> TxInputs {
        // box in boxes.into_iter() {
        //     let _box: chain::ErgoBoxCandidate = jbox.into_serde().unwrap();
        // }
        TxInputs(vec![])
    }
}

/// TODO: copy docs from ErgoBox
#[wasm_bindgen]
pub struct Transaction(chain::Transaction);

#[wasm_bindgen]
impl Transaction {
    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

// TODO: explain wasm_bindgen limitations (supported types)
/// TODO: add doc
#[wasm_bindgen]
pub fn signed_p2pk_transaction(
    _inputs: TxInputs,
    _current_height: u32,
    _recipient: Address,
    _send_change_to: Address,
    _sk: PrivateKey,
) -> Result<Transaction, JsValue> {
    // TODO: create and sign a transaction
    todo!()
}
