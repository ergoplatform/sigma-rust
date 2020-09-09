use sigma_tree::wallet;
use wasm_bindgen::prelude::*;

use crate::{
    address::Address,
    box_coll::{ErgoBoxCandidates, ErgoBoxes},
    box_selector::BoxSelector,
    ergo_box::BoxValue,
    transaction::UnsignedTransaction,
};

#[wasm_bindgen]
pub struct TxBuilder(wallet::tx_builder::TxBuilder);

#[wasm_bindgen]
impl TxBuilder {
    #[wasm_bindgen]
    pub fn new(
        box_selector: BoxSelector,
        inputs: ErgoBoxes,
        output_candidates: ErgoBoxCandidates,
        current_height: u32,
        fee_amount: BoxValue,
    ) -> Result<TxBuilder, JsValue> {
        let _ = box_selector.inner();
        Err(JsValue::from_str("Not yet implemented"))
    }

    #[wasm_bindgen]
    pub fn with_change_sent_to(
        &self,
        change_address: &Address,
        min_change_value: BoxValue,
    ) -> TxBuilder {
        todo!()
    }

    #[wasm_bindgen]
    pub fn build(&self) -> Result<UnsignedTransaction, JsValue> {
        Err(JsValue::from_str("Not yet implemented"))
    }
}
