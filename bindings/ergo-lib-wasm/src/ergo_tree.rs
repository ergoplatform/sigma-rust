//! ErgoTree

use std::convert::TryFrom;

use ergo_lib::chain::Base16DecodedBytes;
use ergo_lib::serialization::SigmaSerializable;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoTree(ergo_lib::ErgoTree);

#[wasm_bindgen]
impl ErgoTree {
    /// Decode from base16 encoded serialized ErgoTree
    pub fn from_base16_bytes(s: &str) -> Result<ErgoTree, JsValue> {
        let bytes = Base16DecodedBytes::try_from(s.to_string())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        ergo_lib::ErgoTree::sigma_parse_bytes(bytes.0)
            .map(ErgoTree)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}
