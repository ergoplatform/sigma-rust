//! Block header

use ergo_lib::ergotree_ir::chain::header::Header;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::error_conversion::to_js;
use ergo_lib::chain::ergo_state_context::Headers;
use std::convert::TryInto;

/// Block header
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct BlockHeader(Header);

#[wasm_bindgen]
impl BlockHeader {
    /// Parse from JSON (Node API)
    pub fn from_json(json: &str) -> Result<BlockHeader, JsValue> {
        serde_json::from_str(json).map(Self).map_err(to_js)
    }
}

/// Collection of BlockHeaders
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeaders(Vec<BlockHeader>);

#[wasm_bindgen]
impl BlockHeaders {
    /// parse BlockHeader array from JSON (Node API)
    #[allow(clippy::boxed_local, clippy::or_fun_call)]
    pub fn from_json(json_vals: Box<[JsValue]>) -> Result<BlockHeaders, JsValue> {
        json_vals
            .iter()
            .try_fold(vec![], |mut acc, jb| {
                let b: Header = if jb.is_string() {
                    let jb_str = jb
                        .as_string()
                        .ok_or(JsValue::from_str("Expected BlockHeader JSON as string"))?;
                    serde_json::from_str(jb_str.as_str())
                } else {
                    jb.into_serde::<ergo_lib::ergotree_ir::chain::header::Header>()
                }
                .map_err(|e| {
                    JsValue::from_str(&format!(
                        "Failed to parse BlockHeader from JSON string: {:?} \n with error: {}",
                        jb, e
                    ))
                })?;
                acc.push(b);
                Ok(acc)
            })
            .map(BlockHeaders::from)
    }

    /// Create new collection with one element
    #[wasm_bindgen(constructor)]
    pub fn new(b: &BlockHeader) -> BlockHeaders {
        BlockHeaders(vec![b.clone()])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Add an element to the collection
    pub fn add(&mut self, b: &BlockHeader) {
        self.0.push(b.clone());
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> BlockHeader {
        self.0[index].clone()
    }
}

impl From<Vec<Header>> for BlockHeaders {
    fn from(bs: Vec<Header>) -> Self {
        BlockHeaders(bs.into_iter().map(BlockHeader::from).collect())
    }
}

impl From<BlockHeaders> for Vec<Header> {
    fn from(bs: BlockHeaders) -> Self {
        bs.0.into_iter().map(Header::from).collect()
    }
}

impl From<BlockHeaders> for Headers {
    fn from(bs: BlockHeaders) -> Self {
        bs.try_into().unwrap()
    }
}
