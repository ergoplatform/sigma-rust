use sigma_tree::chain;
use wasm_bindgen::prelude::*;

use crate::ergo_box::{ErgoBox, ErgoBoxCandidate};

/// Collection of ErgoBox'es
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxes(Vec<chain::ergo_box::ErgoBox>);

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
            .map(ErgoBoxes)
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
}

impl Into<Vec<chain::ergo_box::ErgoBox>> for ErgoBoxes {
    fn into(self) -> Vec<chain::ergo_box::ErgoBox> {
        self.0
    }
}

/// Collection of ErgoBoxCandidates
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxCandidates(Vec<chain::ergo_box::ErgoBoxCandidate>);

#[wasm_bindgen]
impl ErgoBoxCandidates {
    /// Create new outputs
    #[wasm_bindgen(constructor)]
    pub fn new(box_candidate: &ErgoBoxCandidate) -> ErgoBoxCandidates {
        ErgoBoxCandidates(vec![box_candidate.clone().into()])
    }
}

impl Into<Vec<chain::ergo_box::ErgoBoxCandidate>> for ErgoBoxCandidates {
    fn into(self) -> Vec<chain::ergo_box::ErgoBoxCandidate> {
        self.0
    }
}
