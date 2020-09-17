//! Builder for an UnsignedTransaction

// TODO: remove after the implementation
#![allow(unused_variables)]
#![allow(dead_code)]

use box_value::BoxValueError;
use thiserror::Error;

use crate::chain::address::Address;
use crate::chain::ergo_box::box_value;
use crate::chain::{
    ergo_box::ErgoBoxAssets,
    ergo_box::ErgoBoxId,
    ergo_box::{box_value::BoxValue, ErgoBoxCandidate},
    input::UnsignedInput,
    transaction::unsigned::UnsignedTransaction,
};

use super::box_selector::{BoxSelection, BoxSelector, BoxSelectorError};

/// Unsigned transaction builder
pub struct TxBuilder<S: ErgoBoxAssets> {
    box_selector: Box<dyn BoxSelector<S>>,
    boxes_to_spend: Vec<S>,
    output_candidates: Vec<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount: BoxValue,
}

impl<S: ErgoBoxAssets + ErgoBoxId + Clone> TxBuilder<S> {
    /// Creates new TxBuilder
    pub fn new(
        box_selector: Box<dyn BoxSelector<S>>,
        boxes_to_spend: Vec<S>,
        output_candidates: Vec<ErgoBoxCandidate>,
        current_height: u32,
        fee_amount: BoxValue,
    ) -> Result<TxBuilder<S>, TxBuilderError> {
        // TODO: check parameters and return an Err
        Ok(TxBuilder {
            box_selector,
            boxes_to_spend,
            output_candidates,
            current_height,
            fee_amount,
        })
    }

    /// Adds an address to send change to.
    /// if change value is lower than `min_change_value` it will be left to miners
    pub fn with_change_sent_to(
        &self,
        change_address: &Address,
        min_change_value: BoxValue,
    ) -> TxBuilder<S> {
        todo!()
    }

    /// Build the unsigned transaction
    pub fn build(&self) -> Result<UnsignedTransaction, TxBuilderError> {
        let total_output_value: BoxValue =
            box_value::sum(self.output_candidates.iter().map(|b| b.value))?;
        let selection: BoxSelection<S> = self.box_selector.select(
            self.boxes_to_spend.clone(),
            total_output_value,
            vec![].as_slice(),
        )?;
        // let total_input_value = box_value::sum(selection.boxes.iter().map(|b| b.value()))?;
        // TODO: add returning change
        // TODO: miner's fee
        Ok(UnsignedTransaction::new(
            selection
                .boxes
                .into_iter()
                .map(UnsignedInput::from)
                .collect(),
            vec![],
            self.output_candidates.clone(),
        ))
    }
}
/// Errors of TxBuilder
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum TxBuilderError {
    /// Box selection error
    #[error("Box selector error {}", 0)]
    BoxSelectorError(BoxSelectorError),
    /// Box value error
    #[error("Box value error")]
    BoxValueError(BoxValueError),
}

impl From<BoxSelectorError> for TxBuilderError {
    fn from(e: BoxSelectorError) -> Self {
        TxBuilderError::BoxSelectorError(e)
    }
}

impl From<BoxValueError> for TxBuilderError {
    fn from(e: BoxValueError) -> Self {
        TxBuilderError::BoxValueError(e)
    }
}
