use sigma_tree::chain;
use wasm_bindgen::prelude::*;

use crate::ergo_box::ErgoBoxCandidate;

/// Collection of ErgoBox'es
#[wasm_bindgen]
pub struct ErgoBoxes(Vec<chain::ergo_box::ErgoBox>);

#[wasm_bindgen]
impl ErgoBoxes {
    /// parse ErgoBox array from json
    #[allow(clippy::boxed_local)]
    pub fn from_boxes(_boxes: Box<[JsValue]>) -> ErgoBoxes {
        // box in boxes.into_iter() {
        //     let _box: chain::ErgoBoxCandidate = jbox.into_serde().unwrap();
        // }
        ErgoBoxes(vec![])
    }
}

/// Collection of ErgoBoxCandidates
#[wasm_bindgen]
pub struct ErgoBoxCandidates(Vec<chain::ergo_box::ErgoBoxCandidate>);

#[wasm_bindgen]
impl ErgoBoxCandidates {
    /// Create new outputs
    #[wasm_bindgen(constructor)]
    pub fn new(box_candidate: ErgoBoxCandidate) -> ErgoBoxCandidates {
        ErgoBoxCandidates(vec![box_candidate.into()])
    }
}
