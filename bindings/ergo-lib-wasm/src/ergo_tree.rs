//! ErgoTree

use std::convert::TryFrom;

use ergo_lib::chain::Base16DecodedBytes;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::{ast::Constant, error_conversion::to_js};

/// The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoTree(ergo_lib::ergotree_ir::ergo_tree::ErgoTree);

#[wasm_bindgen]
impl ErgoTree {
    /// Decode from base16 encoded serialized ErgoTree
    pub fn from_base16_bytes(s: &str) -> Result<ErgoTree, JsValue> {
        let bytes = Base16DecodedBytes::try_from(s.to_string()).map_err(to_js)?;
        ErgoTree::from_bytes(bytes.0)
    }

    /// Decode from encoded serialized ErgoTree
    pub fn from_bytes(data: Vec<u8>) -> Result<ErgoTree, JsValue> {
        ergo_lib::ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&data)
            .map(ErgoTree)
            .map_err(to_js)
    }
    /// Returns serialized bytes or fails with error if ErgoTree cannot be serialized
    pub fn sigma_serialize_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0.sigma_serialize_bytes().map_err(to_js)
    }

    /// Returns Base16-encoded serialized bytes
    pub fn to_base16_bytes(&self) -> Result<String, JsValue> {
        self.0.to_base16_bytes().map_err(to_js)
    }

    /// Returns constants number as stored in serialized ErgoTree or error if the parsing of
    /// constants is failed
    pub fn constants_len(&self) -> Result<usize, JsValue> {
        self.0.constants_len().map_err(to_js)
    }

    /// Returns constant with given index (as stored in serialized ErgoTree)
    /// or None if index is out of bounds
    /// or error if constants parsing were failed
    pub fn get_constant(&self, index: usize) -> Result<Option<Constant>, JsValue> {
        self.0
            .get_constant(index)
            .map(|opt| opt.map(|c| c.into()))
            .map_err(to_js)
    }

    /// Consumes the calling ErgoTree and returns new ErgoTree with a new constant value
    /// for a given index in constants list (as stored in serialized ErgoTree), or an error.
    /// After the call the calling ErgoTree will be null.
    pub fn with_constant(self, index: usize, constant: &Constant) -> Result<ErgoTree, JsValue> {
        self.0
            .with_constant(index, constant.clone().into())
            .map(ErgoTree)
            .map_err(to_js)
    }

    /// Serialized proposition expression of SigmaProp type with
    /// ConstantPlaceholder nodes instead of Constant nodes
    pub fn template_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0.template_bytes().map_err(to_js)
    }
}
