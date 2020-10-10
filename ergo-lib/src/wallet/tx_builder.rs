//! Builder for an UnsignedTransaction

use box_value::BoxValueError;
use thiserror::Error;

use crate::chain::address::Address;
use crate::chain::address::AddressEncoder;
use crate::chain::address::NetworkPrefix;
use crate::chain::contract::Contract;
use crate::chain::data_input::DataInput;
use crate::chain::ergo_box::box_builder::ErgoBoxCandidateBuilder;
use crate::chain::ergo_box::box_builder::ErgoBoxCandidateBuilderError;
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
    data_inputs: Vec<DataInput>,
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
    ) -> TxBuilder<S> {
        TxBuilder {
            box_selector,
            boxes_to_spend,
            data_inputs: vec![],
            output_candidates,
            current_height,
            fee_amount,
            change_address,
            min_change_value,
        }
    }

    /// Set transaction's data inputs
    pub fn set_data_inputs(mut self, data_inputs: Vec<DataInput>) -> Self {
        self.data_inputs = data_inputs;
        self
    }

    /// Build the unsigned transaction
    pub fn build(self) -> Result<UnsignedTransaction, TxBuilderError> {
        if self.boxes_to_spend.is_empty() {
            return Err(TxBuilderError::InvalidArgs(
                "boxes_to_spend is empty".to_string(),
            ));
        }
        if self.output_candidates.is_empty() {
            return Err(TxBuilderError::InvalidArgs(
                "output_candidates is empty".to_string(),
            ));
        }
        let total_output_value: BoxValue =
            box_value::checked_sum(self.output_candidates.iter().map(|b| b.value))?
                .checked_add(&self.fee_amount)?;
        let selection: BoxSelection<S> = self.box_selector.select(
            self.boxes_to_spend.clone(),
            total_output_value,
            vec![].as_slice(),
        )?;
        let mut output_candidates = self.output_candidates.clone();

        let change_address_ergo_tree = Contract::pay_to_address(&self.change_address)?.ergo_tree();
        let change_boxes: Result<Vec<ErgoBoxCandidate>, ErgoBoxCandidateBuilderError> = selection
            .change_boxes
            .iter()
            .filter(|b| b.value >= self.min_change_value)
            .map(|b| {
                ErgoBoxCandidateBuilder::new(
                    b.value,
                    change_address_ergo_tree.clone(),
                    self.current_height,
                )
                .build()
            })
            .collect();
        output_candidates.append(&mut change_boxes?);
        // add miner's fee
        let miner_fee_box = new_miner_fee_box(self.fee_amount, self.current_height)?;
        output_candidates.push(miner_fee_box);
        Ok(UnsignedTransaction::new(
            selection
                .boxes
                .into_iter()
                .map(UnsignedInput::from)
                .collect(),
            self.data_inputs,
            output_candidates,
        ))
    }
}

/// Create a box with miner's contract and a given value
pub fn new_miner_fee_box(
    fee_amount: BoxValue,
    creation_height: u32,
) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
    let address_encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
    let miner_fee_address = address_encoder
        .parse_address_from_str(MINERS_FEE_MAINNET_ADDRESS)
        .unwrap();
    let ergo_tree = miner_fee_address.script().unwrap();
    ErgoBoxCandidateBuilder::new(fee_amount, ergo_tree, creation_height).build()
}

/// Errors of TxBuilder
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum TxBuilderError {
    /// Box selection error
    #[error("Box selector error: {0}")]
    BoxSelectorError(BoxSelectorError),
    /// Box value error
    #[error("Box value error")]
    BoxValueError(BoxValueError),
    /// Serialization error
    #[error("Serialization error")]
    SerializationError(SerializationError),
    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    /// ErgoBoxCandidate error
    #[error("ErgoBoxCandidateBuilder error: {0}")]
    ErgoBoxCandidateBuilderError(ErgoBoxCandidateBuilderError),
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

impl From<ErgoBoxCandidateBuilderError> for TxBuilderError {
    fn from(e: ErgoBoxCandidateBuilderError) -> Self {
        TxBuilderError::ErgoBoxCandidateBuilderError(e)
    }
}

#[cfg(test)]
mod tests {

    use proptest::{collection::vec, prelude::*};

    use crate::chain::ergo_box::ErgoBox;
    use crate::test_util::force_any_val;
    use crate::wallet::box_selector::simple::SimpleBoxSelector;

    use super::*;

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
            BoxValue::SAFE_USER_MIN,
        );
        assert!(r.build().is_err());
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
            BoxValue::SAFE_USER_MIN,
        );
        assert!(r.build().is_err());
    }

    proptest! {

        #[test]
        fn test_build_tx(inputs in vec(any_with::<ErgoBox>((BoxValue::MIN_RAW * 5000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                         outputs in vec(any_with::<ErgoBoxCandidate>((BoxValue::MIN_RAW * 1000 ..BoxValue::MIN_RAW * 2000).into()), 1..2),
                         change_address in any::<Address>(),
                         miners_fee in any_with::<BoxValue>((BoxValue::MIN_RAW * 100..BoxValue::MIN_RAW * 200).into()),
                         data_inputs in vec(any::<DataInput>(), 0..2)) {
            let min_change_value = BoxValue::SAFE_USER_MIN;

            let all_outputs = box_value::checked_sum(outputs.iter().map(|b| b.value)).unwrap()
                                                                             .checked_add(&miners_fee)
                                                                             .unwrap();
            let all_inputs = box_value::checked_sum(inputs.iter().map(|b| b.value)).unwrap();

            prop_assume!(all_outputs < all_inputs);

            let tx_builder = TxBuilder::new(
                SimpleBoxSelector::new(),
                inputs.clone(),
                outputs.clone(),
                1,
                miners_fee,
                change_address.clone(),
                min_change_value,
            )
            .set_data_inputs(data_inputs.clone());
            let tx = tx_builder.build().unwrap();
            prop_assert!(outputs.into_iter().all(|i| tx.output_candidates.iter().any(|o| *o == i)),
                         "tx.output_candidates is missing some outputs");
            let tx_all_inputs_vals: Vec<BoxValue> = tx.inputs.iter()
                                                             .map(|i| inputs.iter()
                                                                  .find(|ib| ib.box_id() == i.box_id).unwrap().value)
                                                             .collect();
            let tx_all_inputs_sum = box_value::checked_sum(tx_all_inputs_vals.into_iter()).unwrap();
            let expected_change = tx_all_inputs_sum.checked_sub(&all_outputs).unwrap();
            prop_assert!(tx.output_candidates.iter().any(|b| {
                b.value == expected_change && b.ergo_tree == change_address.script().unwrap()
            }), "box with change {:?} is not found in outputs: {:?}", expected_change, tx.output_candidates);
            prop_assert!(tx.output_candidates.iter().any(|b| {
                b.value == miners_fee
            }), "box with miner's fee {:?} is not found in outputs: {:?}", miners_fee, tx.output_candidates);
            prop_assert_eq!(tx.data_inputs, data_inputs, "unexpected data inputs");
        }
    }
}
