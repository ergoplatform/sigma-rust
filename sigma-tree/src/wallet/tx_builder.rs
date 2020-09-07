//! Builder for an UnsignedTransaction

// TODO: remove after the implementation
#![allow(unused_variables)]

use crate::chain::{
    address::Address,
    ergo_box::{box_value::BoxValue, ErgoBox, ErgoBoxCandidate},
    token::TokenAmount,
    transaction::unsigned::UnsignedTransaction,
};

// TODO: extract

/// Assets that ErgoBox holds
pub trait ErgoBoxAssets {
    /// Box value
    fn value(&self) -> BoxValue;
    /// Tokens (ids and amounts)
    fn tokens(&self) -> &[TokenAmount];
}

// TODO: extract

/// Selected boxes (by [`BoxSelector`])
pub struct BoxSelection<T: ErgoBoxAssets> {
    /// selected boxes to spend
    pub boxes: Vec<T>,
    /// box assets with returning change amounts (to be put in tx outputs)
    pub change_boxes: Vec<T>,
}

/// Box selector
pub trait BoxSelector {
    /// Selects boxes out of the provided inputs to satisfy target balance and tokens
    fn select<T: ErgoBoxAssets>(
        inputs: &[T],
        target_balance: BoxValue,
        target_tokens: &[TokenAmount],
    ) -> Result<BoxSelection<T>, BoxSelectorError> {
        todo!()
    }
}

/// Errors of BoxSelector
pub enum BoxSelectorError {}

/// Unsigned transaction builder
pub struct TxBuilder {}

impl TxBuilder {
    /// Creates new TxBuilder
    pub fn new<T: BoxSelector>(
        box_selector: T,
        inputs: &[ErgoBox],
        output_candidates: &[ErgoBoxCandidate],
        current_height: u32,
        fee_amount: BoxValue,
    ) -> TxBuilder {
        todo!()
    }

    /// Adds an address to send change to.
    /// if change value is lower than `min_change_value` it will be left to miners
    pub fn with_change_sent_to(
        &self,
        change_address: &Address,
        min_change_value: BoxValue,
    ) -> TxBuilder {
        todo!()
    }

    /// Build the unsigned transaction
    pub fn build(&self) -> UnsignedTransaction {
        todo!()
    }
}

/// Errors of TxBuilder
pub enum TxBuilderError {}
