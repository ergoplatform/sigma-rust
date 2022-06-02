//! Block header

use ergo_lib::ergo_chain_types::Header;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::error_conversion::to_js;
use ergo_lib::chain::ergo_state_context::Headers;
use std::convert::{TryFrom, TryInto};

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

    /// Get Header's id
    pub fn id(&self) -> BlockId {
        self.0.id.clone().into()
    }
}

/// Block id
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct BlockId(pub(crate) ergo_lib::ergo_chain_types::BlockId);

#[wasm_bindgen]
impl BlockId {
    /// Parse from base 16 encoded string
    pub fn from_str(id: &str) -> Result<BlockId, JsValue> {
        ergo_lib::ergo_chain_types::Digest32::try_from(String::from(id))
            .map(|d| BlockId(ergo_lib::ergo_chain_types::BlockId(d)))
            .map_err(to_js)
    }

    /// Equality check
    pub fn equals(&self, id: &BlockId) -> bool {
        self.0 == id.0
    }
}

/// Collection of BlockHeaders
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeaders(pub(crate) Vec<BlockHeader>);

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
                    jb.into_serde::<ergo_lib::ergo_chain_types::Header>()
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

impl TryFrom<BlockHeaders> for Headers {
    type Error = JsValue;
    fn try_from(bs: BlockHeaders) -> Result<Self, Self::Error> {
        let headers: Vec<Header> = bs.0.into_iter().map(Header::from).collect();
        if headers.len() == 10 {
            #[allow(clippy::unwrap_used)]
            Ok(headers.try_into().unwrap())
        } else {
            Err(js_sys::Error::new(&format!(
                "Incorrect number of block headers, expected 10 but got {}",
                headers.len()
            ))
            .into())
        }
    }
}
