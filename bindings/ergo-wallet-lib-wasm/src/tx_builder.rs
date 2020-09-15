use sigma_tree::{chain::ergo_box::ErgoBox, wallet};
use wallet::box_selector::select_all::SelectAllBoxSelector;
use wasm_bindgen::prelude::*;

use crate::{
    address::Address,
    box_coll::{ErgoBoxCandidates, ErgoBoxes},
    ergo_box::BoxValue,
    transaction::UnsignedTransaction,
};

#[wasm_bindgen]
pub struct TxBuilder(wallet::tx_builder::TxBuilder<SelectAllBoxSelector<ErgoBox>, ErgoBox>);

#[wasm_bindgen]
impl TxBuilder {
    #[wasm_bindgen]
    pub fn new(
        inputs: &ErgoBoxes,
        output_candidates: &ErgoBoxCandidates,
        current_height: u32,
        fee_amount: &BoxValue,
    ) -> Result<TxBuilder, JsValue> {
        sigma_tree::wallet::tx_builder::TxBuilder::new(
            wallet::box_selector::select_all::SelectAllBoxSelector::<ErgoBox>::new(),
            inputs.clone().into(),
            output_candidates.clone().into(),
            current_height,
            fee_amount.clone().into(),
        )
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
        .map(TxBuilder)
    }

    #[wasm_bindgen]
    pub fn with_change_sent_to(
        &self,
        change_address: &Address,
        min_change_value: &BoxValue,
    ) -> TxBuilder {
        todo!()
    }

    #[wasm_bindgen]
    pub fn build(&self) -> Result<UnsignedTransaction, JsValue> {
        self.0
            .build()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(UnsignedTransaction::from)
    }
}
