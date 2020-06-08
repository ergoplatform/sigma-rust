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
pub struct Address(Box<dyn chain::Address>);

#[wasm_bindgen]
impl Address {
    /// Decode(base58) address
    pub fn from_testnet_str(s: &str) -> Result<Address, JsValue> {
        chain::AddressEncoder::new(chain::NetworkPrefix::Testnet)
            .parse_address_from_str(s)
            .map(|a| Address(a))
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

/// TODO: wrap sigma-tree type
#[wasm_bindgen]
pub struct PrivateKey(String);

#[wasm_bindgen]
impl PrivateKey {
    /// Decode from string
    pub fn from_str(_: &str) -> PrivateKey {
        PrivateKey(String::new())
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

/// Transaction outputs
#[wasm_bindgen]
pub struct TxOutputs(Vec<chain::ErgoBoxCandidate>);

#[wasm_bindgen]
impl TxOutputs {
    /// parse ErgoBoxCandidate from json
    pub fn from_boxes(_boxes: Box<[JsValue]>) -> TxOutputs {
        // box in boxes.into_iter() {
        //     let _box: chain::ErgoBoxCandidate = jbox.into_serde().unwrap();
        // }
        TxOutputs(vec![])
    }
}

/// TODO: copy docs from ErgoBox
#[wasm_bindgen]
pub struct ErgoBoxCandidate(chain::ErgoBoxCandidate);

#[wasm_bindgen]
impl ErgoBoxCandidate {
    /// make new box
    #[wasm_bindgen(constructor)]
    pub fn new(value: u64, creation_height: u32, contract: Contract) -> ErgoBoxCandidate {
        let b = chain::ErgoBoxCandidate::new(value, contract.0.get_ergo_tree(), creation_height);
        ErgoBoxCandidate(b)
    }

    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

/// TODO: docs
#[wasm_bindgen]
pub struct Contract(chain::Contract);

#[wasm_bindgen]
impl Contract {
    /// send ERGs to the recipient address
    pub fn pay_2pk(_recipient: Address) -> Contract {
        todo!()
    }
}

/// TODO: copy docs from Transaction
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
pub fn new_signed_transaction(
    _inputs: TxInputs,
    _outputs: TxOutputs,
    _send_change_to: Address,
    _sk: PrivateKey,
) -> Result<Transaction, JsValue> {
    // TODO: create and sign a transaction
    Err(JsValue::from_str("Error!"))
}
