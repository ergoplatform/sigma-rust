//! Builder for an UnsignedTransaction

// TODO: remove after the implementation
#![allow(unused_variables)]

use crate::chain::{
    address::Address,
    ergo_box::{box_value::BoxValue, ErgoBox, ErgoBoxCandidate},
    transaction::unsigned::UnsignedTransaction,
};

use super::box_selector::BoxSelector;

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
    ) -> Result<TxBuilder, TxBuilderError> {
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
    pub fn build(&self) -> Result<UnsignedTransaction, TxBuilderError> {
        todo!()
    }
}

/// Errors of TxBuilder
pub enum TxBuilderError {}
