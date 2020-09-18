//! Builder for an UnsignedTransaction

// TODO: remove after the implementation
#![allow(unused_variables)]
#![allow(dead_code)]

use box_value::BoxValueError;
use thiserror::Error;

use crate::chain::address::Address;
use crate::chain::contract::Contract;
use crate::chain::ergo_box::box_value;
use crate::chain::{
    ergo_box::ErgoBoxAssets,
    ergo_box::ErgoBoxId,
    ergo_box::{box_value::BoxValue, ErgoBoxCandidate},
    input::UnsignedInput,
    transaction::unsigned::UnsignedTransaction,
};
use crate::serialization::SerializationError;

use super::box_selector::{BoxSelection, BoxSelector, BoxSelectorError};

/// Unsigned transaction builder
pub struct TxBuilder<S: ErgoBoxAssets> {
    box_selector: Box<dyn BoxSelector<S>>,
    boxes_to_spend: Vec<S>,
    output_candidates: Vec<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount: BoxValue,
    change_address: Option<Address>,
    min_change_value: Option<BoxValue>,
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
            change_address: None,
            min_change_value: None,
        })
    }

    /// Adds an address to send change to.
    /// if change value is lower than `min_change_value` it will be left to miners
    pub fn with_change_sent_to(&mut self, change_address: &Address, min_change_value: BoxValue) {
        // TODO: use in WASM smoke tests when its implemented
        self.change_address = Some(change_address.clone());
        self.min_change_value = Some(min_change_value);
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
        let mut output_candidates = self.output_candidates.clone();
        let total_input_value = box_value::sum(selection.boxes.iter().map(|b| b.value()))?;
        if total_output_value < total_input_value {
            if let Some(change_address) = &self.change_address {
                let change_value = total_input_value.checked_sub(total_output_value)?;
                if let Some(min_change_value) = self.min_change_value {
                    // add returning change (if enough, otherwise give to miners)
                    if min_change_value <= change_value {
                        let tree = Contract::pay_to_address(change_address)?.get_ergo_tree();
                        let change_box =
                            ErgoBoxCandidate::new(change_value, tree, self.current_height);
                        output_candidates.push(change_box);
                    }
                }
            }
        }
        // TODO: miner's fee
        Ok(UnsignedTransaction::new(
            selection
                .boxes
                .into_iter()
                .map(UnsignedInput::from)
                .collect(),
            vec![],
            output_candidates,
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
    /// Serialization error
    #[error("Serialization error")]
    SerializationError(SerializationError),
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

impl From<SerializationError> for TxBuilderError {
    fn from(e: SerializationError) -> Self {
        TxBuilderError::SerializationError(e)
    }
}
