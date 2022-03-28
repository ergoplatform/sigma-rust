//! Box collection types
use ergo_lib::ergotree_ir::chain;
use wasm_bindgen::prelude::*;

use crate::ergo_box::{ErgoBox, ErgoBoxCandidate};

/// Collection of ErgoBox'es
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxes(pub(crate) Vec<ErgoBox>);

#[wasm_bindgen]
impl ErgoBoxes {
    /// parse ErgoBox array from json
    #[allow(clippy::boxed_local, clippy::or_fun_call)]
    pub fn from_boxes_json(json_vals: Box<[JsValue]>) -> Result<ErgoBoxes, JsValue> {
        json_vals
            .iter()
            .try_fold(vec![], |mut acc, jb| {
                let b: chain::ergo_box::ErgoBox = if jb.is_string() {
                    let jb_str = jb
                        .as_string()
                        .ok_or(JsValue::from_str("Expected ErgoBox JSON as string"))?;
                    serde_json::from_str(jb_str.as_str())
                } else {
                    jb.into_serde::<chain::ergo_box::ErgoBox>()
                }
                .map_err(|e| {
                    JsValue::from_str(&format!(
                        "Failed to parse ErgoBox from JSON string: {:?} \n with error: {}",
                        jb, e
                    ))
                })?;
                acc.push(b);
                Ok(acc)
            })
            .map(ErgoBoxes::from)
    }

    /// Create new collection with one element
    #[wasm_bindgen(constructor)]
    pub fn new(b: &ErgoBox) -> ErgoBoxes {
        ErgoBoxes(vec![b.clone()])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Add an element to the collection
    pub fn add(&mut self, b: &ErgoBox) {
        self.0.push(b.clone());
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> ErgoBox {
        self.0[index].clone()
    }

    /// Empty ErgoBoxes
    pub fn empty() -> ErgoBoxes {
        ErgoBoxes(vec![])
    }
}

impl From<Vec<chain::ergo_box::ErgoBox>> for ErgoBoxes {
    fn from(bs: Vec<chain::ergo_box::ErgoBox>) -> Self {
        ErgoBoxes(bs.into_iter().map(ErgoBox::from).collect())
    }
}

impl From<ErgoBoxes> for Vec<chain::ergo_box::ErgoBox> {
    fn from(bs: ErgoBoxes) -> Self {
        bs.0.into_iter()
            .map(chain::ergo_box::ErgoBox::from)
            .collect()
    }
}

/// Collection of ErgoBoxCandidates
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxCandidates(Vec<ErgoBoxCandidate>);

#[wasm_bindgen]
impl ErgoBoxCandidates {
    /// Create new outputs
    #[wasm_bindgen(constructor)]
    pub fn new(box_candidate: &ErgoBoxCandidate) -> ErgoBoxCandidates {
        ErgoBoxCandidates(vec![box_candidate.clone()])
    }

    /// sometimes it's useful to keep track of an empty list
    /// but keep in mind Ergo transactions need at least 1 output
    pub fn empty() -> ErgoBoxCandidates {
        ErgoBoxCandidates(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> ErgoBoxCandidate {
        self.0[index].clone()
    }

    /// Add an element to the collection
    pub fn add(&mut self, b: &ErgoBoxCandidate) {
        self.0.push(b.clone());
    }
}

impl From<ErgoBoxCandidates> for Vec<chain::ergo_box::ErgoBoxCandidate> {
    fn from(v: ErgoBoxCandidates) -> Self {
        v.0.iter().map(|i| i.clone().into()).collect()
    }
}

impl From<Vec<chain::ergo_box::ErgoBoxCandidate>> for ErgoBoxCandidates {
    fn from(v: Vec<chain::ergo_box::ErgoBoxCandidate>) -> Self {
        ErgoBoxCandidates(v.into_iter().map(ErgoBoxCandidate::from).collect())
    }
}

impl From<&ErgoBoxCandidates> for Vec<chain::ergo_box::ErgoBoxCandidate> {
    fn from(v: &ErgoBoxCandidates) -> Self {
        v.0.clone().iter().map(|i| i.clone().into()).collect()
    }
}
