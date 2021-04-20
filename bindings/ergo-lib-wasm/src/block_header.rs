//! Block header
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// Block header
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct BlockHeader(ergo_lib::chain::block_header::BlockHeader);

#[wasm_bindgen]
impl BlockHeader {
    /// Parse from JSON (Node API)
    pub fn from_json(json: &str) -> Result<BlockHeader, JsValue> {
        serde_json::from_str(json)
            .map(Self)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

/// Collection of BlockHeaders
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeaders(Vec<BlockHeader>);

#[wasm_bindgen]
impl BlockHeaders {
    /// parse BlockHeader array from json
    #[allow(clippy::boxed_local, clippy::or_fun_call)]
    pub fn from_json(boxes: Box<[JsValue]>) -> Result<BlockHeaders, JsValue> {
        boxes
            .iter()
            .try_fold(vec![], |mut acc, jb| {
                let b: chain::block_header::BlockHeader = if jb.is_string() {
                    let jb_str = jb
                        .as_string()
                        .ok_or(JsValue::from_str("Expected BlockHeader JSON as string"))?;
                    serde_json::from_str(jb_str.as_str())
                } else {
                    jb.into_serde::<chain::block_header::BlockHeader>()
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

impl From<Vec<chain::block_header::BlockHeader>> for BlockHeaders {
    fn from(bs: Vec<chain::block_header::BlockHeader>) -> Self {
        BlockHeaders(bs.into_iter().map(BlockHeader::from).collect())
    }
}

impl From<BlockHeaders> for Vec<chain::block_header::BlockHeader> {
    fn from(bs: BlockHeaders) -> Self {
        bs.0.into_iter()
            .map(chain::block_header::BlockHeader::from)
            .collect()
    }
}
