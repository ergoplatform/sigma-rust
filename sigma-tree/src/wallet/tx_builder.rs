//! Builder for an UnsignedTransaction

// TODO: remove after the implementation
#![allow(unused_variables)]
#![allow(dead_code)]

use std::convert::TryInto;

use thiserror::Error;

use crate::chain::{
    address::Address,
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
        let selection: BoxSelection<S> = self.box_selector.select(
            self.boxes_to_spend.clone(),
            1u64.try_into().unwrap(),
            vec![].as_slice(),
        )?;
        // TODO: add returning change
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
}

impl From<BoxSelectorError> for TxBuilderError {
    fn from(e: BoxSelectorError) -> Self {
        TxBuilderError::BoxSelectorError(e)
    }
}
