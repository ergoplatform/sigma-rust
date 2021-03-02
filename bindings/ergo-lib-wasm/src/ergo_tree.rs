//! ErgoTree

use std::convert::TryFrom;

use ergo_lib::chain::Base16DecodedBytes;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoTree(ergo_lib::ergotree_ir::ergo_tree::ErgoTree);

#[wasm_bindgen]
impl ErgoTree {
    /// Decode from base16 encoded serialized ErgoTree
    pub fn from_base16_bytes(s: &str) -> Result<ErgoTree, JsValue> {
        let bytes = Base16DecodedBytes::try_from(s.to_string())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        ErgoTree::from_bytes(bytes.0)
    }

    /// Decode from encoded serialized ErgoTree
    pub fn from_bytes(data: Vec<u8>) -> Result<ErgoTree, JsValue> {
        ergo_lib::ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(data)
            .map(ErgoTree)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
    /// Encode Ergo tree as serialized bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.sigma_serialize_bytes()
    }
}
