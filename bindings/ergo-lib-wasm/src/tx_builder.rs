//! Unsigned transaction builder
use ergo_lib::wallet;
use wasm_bindgen::prelude::*;

use crate::box_selector::BoxSelection;
use crate::context_extension::ContextExtension;
use crate::data_input::DataInputs;
use crate::ergo_box::BoxId;
use crate::error_conversion::to_js;
use crate::token::Tokens;
use crate::{
    address::Address, box_coll::ErgoBoxCandidates, ergo_box::BoxValue,
    transaction::UnsignedTransaction,
};

/// Unsigned transaction builder
#[wasm_bindgen]
pub struct TxBuilder(
    wallet::tx_builder::TxBuilder<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox>,
);

#[wasm_bindgen]
impl TxBuilder {
    /// Suggested transaction fee (semi-default value used across wallets and dApps as of Oct 2020)
    #[allow(non_snake_case)]
    pub fn SUGGESTED_TX_FEE() -> BoxValue {
        BoxValue(wallet::tx_builder::SUGGESTED_TX_FEE())
    }

    /// Creates new TxBuilder
    /// `box_selection` - selected input boxes (via [`super::box_selector`])
    /// `output_candidates` - output boxes to be "created" in this transaction,
    /// `current_height` - chain height that will be used in additionally created boxes (change, miner's fee, etc.),
    /// `fee_amount` - miner's fee,
    /// `change_address` - change (inputs - outputs) will be sent to this address,
    /// will be given to miners,
    pub fn new(
        box_selection: &BoxSelection,
        output_candidates: &ErgoBoxCandidates,
        current_height: u32,
        fee_amount: &BoxValue,
        change_address: &Address,
    ) -> TxBuilder {
        TxBuilder(ergo_lib::wallet::tx_builder::TxBuilder::new(
            box_selection.clone().into(),
            output_candidates.clone().into(),
            current_height,
            fee_amount.clone().into(),
            change_address.clone().into(),
        ))
    }

    /// Set transaction's data inputs
    pub fn set_data_inputs(&mut self, data_inputs: &DataInputs) {
        self.0.set_data_inputs(data_inputs.into())
    }

    /// Set context extension for a given input
    pub fn set_context_extension(&mut self, box_id: &BoxId, context_extension: &ContextExtension) {
        self.0
            .set_context_extension(box_id.clone().into(), context_extension.clone().into());
    }

    /// Permits the burn of the given token amount, i.e. allows this token amount to be omitted in the outputs
    pub fn set_token_burn_permit(&mut self, tokens: &Tokens) {
        self.0
            .set_token_burn_permit(tokens.clone().0.into_iter().map(|t| t.into()).collect())
    }

    /// Build the unsigned transaction
    pub fn build(&self) -> Result<UnsignedTransaction, JsValue> {
        self.0
            .clone()
            .build()
            .map_err(to_js)
            .map(UnsignedTransaction::from)
    }

    /// Get box selection
    pub fn box_selection(&self) -> BoxSelection {
        self.0.box_selection().into()
    }

    /// Get data inputs
    pub fn data_inputs(&self) -> DataInputs {
        self.0.data_inputs().into()
    }

    /// Get outputs EXCLUDING fee and change
    pub fn output_candidates(&self) -> ErgoBoxCandidates {
        self.0.output_candidates().into()
    }

    /// Get current height
    pub fn current_height(&self) -> u32 {
        self.0.current_height()
    }

    /// Get fee amount
    pub fn fee_amount(&self) -> BoxValue {
        self.0.fee_amount().into()
    }

    /// Get change address
    pub fn change_address(&self) -> Address {
        self.0.change_address().into()
    }
}
