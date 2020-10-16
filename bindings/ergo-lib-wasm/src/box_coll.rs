//! Box collection types
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

use crate::ergo_box::{ErgoBox, ErgoBoxCandidate};

/// Collection of ErgoBox'es
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxes(Vec<ErgoBox>);

#[wasm_bindgen]
impl ErgoBoxes {
    /// parse ErgoBox array from json
    #[allow(clippy::boxed_local, clippy::or_fun_call)]
    pub fn from_boxes_json(boxes: Box<[JsValue]>) -> Result<ErgoBoxes, JsValue> {
        boxes
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
        ErgoBoxes(vec![b.clone().into()])
    }

    /// Add an element to the collection
    pub fn add(&mut self, b: &ErgoBox) {
        self.0.push(b.clone().into());
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> ErgoBox {
        self.0[index].clone()
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
        ErgoBoxCandidates(vec![box_candidate.clone().into()])
    }
}

impl From<ErgoBoxCandidates> for Vec<chain::ergo_box::ErgoBoxCandidate> {
    fn from(v: ErgoBoxCandidates) -> Self {
        v.0.iter().map(|i| i.clone().into()).collect()
    }
}
