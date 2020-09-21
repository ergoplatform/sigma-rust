//! Builder for an UnsignedTransaction

// TODO: remove after the implementation
#![allow(unused_variables)]
#![allow(dead_code)]

use box_value::BoxValueError;
use thiserror::Error;

use crate::chain::address::Address;
use crate::chain::address::AddressEncoder;
use crate::chain::address::NetworkPrefix;
use crate::chain::contract::Contract;
use crate::chain::ergo_box::box_value;
use crate::chain::{
    ergo_box::ErgoBoxAssets,
    ergo_box::ErgoBoxId,
    ergo_box::{box_value::BoxValue, ErgoBoxCandidate},
    input::UnsignedInput,
    transaction::unsigned::UnsignedTransaction,
};
use crate::constants::MINERS_FEE_MAINNET_ADDRESS;
use crate::serialization::SerializationError;

use super::box_selector::{BoxSelection, BoxSelector, BoxSelectorError};

/// Unsigned transaction builder
pub struct TxBuilder<S: ErgoBoxAssets> {
    box_selector: Box<dyn BoxSelector<S>>,
    boxes_to_spend: Vec<S>,
    output_candidates: Vec<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount: BoxValue,
    change_address: Address,
    min_change_value: BoxValue,
}

impl<S: ErgoBoxAssets + ErgoBoxId + Clone> TxBuilder<S> {
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
        box_selector: Box<dyn BoxSelector<S>>,
        boxes_to_spend: Vec<S>,
        output_candidates: Vec<ErgoBoxCandidate>,
        current_height: u32,
        fee_amount: BoxValue,
        change_address: Address,
        min_change_value: BoxValue,
    ) -> Result<TxBuilder<S>, TxBuilderError> {
        if boxes_to_spend.is_empty() {
            return Err(TxBuilderError::InvalidArgs(
                "boxes_to_spend is empty".to_string(),
            ));
        }
        if output_candidates.is_empty() {
            return Err(TxBuilderError::InvalidArgs(
                "output_candidates is empty".to_string(),
            ));
        }
        Ok(TxBuilder {
            box_selector,
            boxes_to_spend,
            output_candidates,
            current_height,
            fee_amount,
            change_address,
            min_change_value,
        })
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

        let change_address_ergo_tree =
            Contract::pay_to_address(&self.change_address)?.get_ergo_tree();
        let mut change_boxes: Vec<ErgoBoxCandidate> = selection
            .change_boxes
            .iter()
            .filter(|b| b.value >= self.min_change_value)
            .map(|b| {
                ErgoBoxCandidate::new(
                    b.value,
                    change_address_ergo_tree.clone(),
                    self.current_height,
                )
            })
            .collect();
        output_candidates.append(&mut change_boxes);
        // add miner's fee
        output_candidates.push(new_miner_fee_box(self.fee_amount, self.current_height));
        Ok(UnsignedTransaction::new(
            selection
                .boxes
                .into_iter()
                .map(UnsignedInput::from)
                .collect(),
            vec![],
            output_candidates,
        ))
        // TODO: add tests
    }
}

fn new_miner_fee_box(fee_amount: BoxValue, creation_height: u32) -> ErgoBoxCandidate {
    let address_encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
    let miner_fee_address = address_encoder
        .parse_address_from_str(MINERS_FEE_MAINNET_ADDRESS)
        .unwrap();
    let ergo_tree = miner_fee_address.script().unwrap();
    ErgoBoxCandidate::new(fee_amount, ergo_tree, creation_height)
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

    /// Invalid arguments
    #[error("Invalid arguments {}", 0)]
    InvalidArgs(String),
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

#[cfg(test)]
mod tests {

    use std::convert::TryInto;

    use proptest::prelude::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    use crate::chain::ergo_box::ErgoBox;
    use crate::wallet::box_selector::simple::SimpleBoxSelector;

    use super::*;

    fn force_any_val<T: Arbitrary>() -> T {
        let mut runner = TestRunner::default();
        any::<T>().new_tree(&mut runner).unwrap().current()
    }

    #[test]
    fn test_empty_inputs() {
        let inputs: Vec<ErgoBox> = vec![];
        let r = TxBuilder::new(
            SimpleBoxSelector::new(),
            inputs,
            vec![force_any_val::<ErgoBoxCandidate>()],
            1,
            force_any_val::<BoxValue>(),
            force_any_val::<Address>(),
            1u64.try_into().unwrap(),
        );
        assert!(r.is_err());
    }

    #[test]
    fn test_empty_outputs() {
        let outputs: Vec<ErgoBoxCandidate> = vec![];
        let r = TxBuilder::new(
            SimpleBoxSelector::new(),
            vec![force_any_val::<ErgoBox>()],
            outputs,
            1,
            force_any_val::<BoxValue>(),
            force_any_val::<Address>(),
            1u64.try_into().unwrap(),
        );
        assert!(r.is_err());
    }
}
