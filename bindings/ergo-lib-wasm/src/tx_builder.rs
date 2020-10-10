//! Unsigned transaction builder
use ergo_lib::{chain, wallet};
use wasm_bindgen::prelude::*;

use crate::data_input::DataInputs;
use crate::{
    address::Address,
    box_coll::{ErgoBoxCandidates, ErgoBoxes},
    box_selector::BoxSelector,
    ergo_box::BoxValue,
    transaction::UnsignedTransaction,
};

/// Unsigned transaction builder
#[wasm_bindgen]
pub struct TxBuilder(wallet::tx_builder::TxBuilder<chain::ergo_box::ErgoBox>);

#[wasm_bindgen]
impl TxBuilder {
    /// Creates new TxBuilder
    /// `box_selector` - input box selection algorithm to choose inputs from `boxes_to_spend`,
    /// `boxes_to_spend` - spendable boxes,
    /// `output_candidates` - output boxes to be "created" in this transaction,
    /// `current_height` - chain height that will be used in additionally created boxes (change, miner's fee, etc.),
    /// `fee_amount` - miner's fee,
    /// `change_address` - change (inputs - outputs) will be sent to this address,
    /// `min_change_value` - minimal value of the change to be sent to `change_address`, value less than that
    /// will be given to miners,
    pub fn new(
        box_selector: BoxSelector,
        inputs: &ErgoBoxes,
        output_candidates: &ErgoBoxCandidates,
        current_height: u32,
        fee_amount: &BoxValue,
        change_address: &Address,
        min_change_value: &BoxValue,
    ) -> TxBuilder {
        TxBuilder(ergo_lib::wallet::tx_builder::TxBuilder::new(
            box_selector.inner::<chain::ergo_box::ErgoBox>(),
            inputs.clone().into(),
            output_candidates.clone().into(),
            current_height,
            fee_amount.clone().into(),
            change_address.clone().into(),
            min_change_value.clone().into(),
        ))
    }

    /// Set transaction's data inputs
    pub fn set_data_inputs(self, data_inputs: &DataInputs) -> TxBuilder {
        TxBuilder(self.0.set_data_inputs(data_inputs.into()))
    }

    /// Build the unsigned transaction
    pub fn build(self) -> Result<UnsignedTransaction, JsValue> {
        self.0
            .build()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(UnsignedTransaction::from)
    }
}
