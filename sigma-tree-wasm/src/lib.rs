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

mod misc;
mod utils;

pub use misc::*;

// use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// TODO: wrap sigma-tree type
#[wasm_bindgen]
pub struct Address(String);

/// TODO: wrap sigma-tree type
#[wasm_bindgen]
pub struct PrivateKey(String);

/// TODO: explain wasm_bindgen limitations (supported types)
/// TODO: add doc
#[wasm_bindgen]
pub fn signed_p2pk_tx(
    inputs: Box<[JsValue]>,
    _current_height: u32,
    _recipient: Address,
    _sk: PrivateKey,
) -> Result<JsValue, JsValue> {
    for jbox in inputs.into_iter() {
        let _box: chain::ErgoBoxCandidate = jbox.into_serde().unwrap();
    }

    let tx = chain::Transaction {
        inputs: vec![],
        data_inputs: vec![],
        outputs: vec![],
    };
    JsValue::from_serde(&tx)
}
