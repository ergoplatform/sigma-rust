use sigma_tree::{chain::ergo_box::ErgoBox, wallet};
use wasm_bindgen::prelude::*;

use crate::{
    address::Address,
    box_coll::{ErgoBoxCandidates, ErgoBoxes},
    box_selector::BoxSelector,
    ergo_box::BoxValue,
    transaction::UnsignedTransaction,
};

#[wasm_bindgen]
pub struct TxBuilder(wallet::tx_builder::TxBuilder<ErgoBox>);

#[wasm_bindgen]
impl TxBuilder {
    pub fn new(
        box_selector: BoxSelector,
        inputs: &ErgoBoxes,
        output_candidates: &ErgoBoxCandidates,
        current_height: u32,
        fee_amount: &BoxValue,
    ) -> Result<TxBuilder, JsValue> {
        sigma_tree::wallet::tx_builder::TxBuilder::new(
            box_selector.inner::<ErgoBox>(),
            inputs.clone().into(),
            output_candidates.clone().into(),
            current_height,
            fee_amount.clone().into(),
        )
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
        .map(TxBuilder)
    }

    pub fn with_change_sent_to(
        &self,
        change_address: &Address,
        min_change_value: &BoxValue,
    ) -> TxBuilder {
        todo!()
    }

    pub fn build(&self) -> Result<UnsignedTransaction, JsValue> {
        self.0
            .build()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(UnsignedTransaction::from)
    }
}
