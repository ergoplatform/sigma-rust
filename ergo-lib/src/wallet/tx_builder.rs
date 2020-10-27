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

use super::box_selector::{BoxSelection, BoxSelectorError};

/// Unsigned transaction builder
pub struct TxBuilder<S: ErgoBoxAssets> {
    box_selection: BoxSelection<S>,
    data_inputs: Vec<DataInput>,
    output_candidates: Vec<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount: BoxValue,
    change_address: Address,
    min_change_value: BoxValue,
}

impl<S: ErgoBoxAssets + ErgoBoxId + Clone> TxBuilder<S> {
    /// Creates new TxBuilder
    /// `box_selection` - selected input boxes  (via [`BoxSelector`])
    /// `output_candidates` - output boxes to be "created" in this transaction,
    /// `current_height` - chain height that will be used in additionally created boxes (change, miner's fee, etc.),
    /// `fee_amount` - miner's fee,
    /// `change_address` - change (inputs - outputs) will be sent to this address,
    /// `min_change_value` - minimal value of the change to be sent to `change_address`, value less than that
    /// will be given to miners,
    pub fn new(
        box_selection: BoxSelection<S>,
        output_candidates: Vec<ErgoBoxCandidate>,
        current_height: u32,
        fee_amount: BoxValue,
        change_address: Address,
        min_change_value: BoxValue,
    ) -> TxBuilder<S> {
        TxBuilder {
            box_selection,
            data_inputs: vec![],
            output_candidates,
            current_height,
            fee_amount,
            change_address,
            min_change_value,
        }
    }

    /// Set transaction's data inputs
    pub fn set_data_inputs(&mut self, data_inputs: Vec<DataInput>) {
        self.data_inputs = data_inputs;
    }

    /// Build the unsigned transaction
    pub fn build(self) -> Result<UnsignedTransaction, TxBuilderError> {
        if self.box_selection.boxes.is_empty() {
            return Err(TxBuilderError::InvalidArgs("inputs is empty".to_string()));
        }
        if self.output_candidates.is_empty() {
            return Err(TxBuilderError::InvalidArgs(
                "output_candidates is empty".to_string(),
            ));
        }
        let mut output_candidates = self.output_candidates.clone();

        let change_address_ergo_tree = Contract::pay_to_address(&self.change_address)?.ergo_tree();
        let change_boxes: Result<Vec<ErgoBoxCandidate>, ErgoBoxCandidateBuilderError> = self
            .box_selection
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
            self.box_selection
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
    BoxSelectorError(#[from] BoxSelectorError),
    /// Box value error
    #[error("Box value error")]
    BoxValueError(#[from] BoxValueError),
    /// Serialization error
    #[error("Serialization error")]
    SerializationError(#[from] SerializationError),
    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    /// ErgoBoxCandidate error
    #[error("ErgoBoxCandidateBuilder error: {0}")]
    ErgoBoxCandidateBuilderError(#[from] ErgoBoxCandidateBuilderError),
}

#[cfg(test)]
mod tests {

    use std::convert::TryInto;

    use proptest::{collection::vec, prelude::*};

    use crate::chain::ergo_box::register::NonMandatoryRegisters;
    use crate::chain::ergo_box::ErgoBox;
    use crate::chain::token::Token;
    use crate::chain::token::TokenId;
    use crate::chain::transaction::TxId;
    use crate::test_util::force_any_val;
    use crate::wallet::box_selector::BoxSelector;
    use crate::wallet::box_selector::SimpleBoxSelector;
    use crate::ErgoTree;

    use super::*;

    #[test]
    fn test_empty_inputs() {
        let box_selection: BoxSelection<ErgoBox> = BoxSelection {
            boxes: vec![],
            change_boxes: vec![],
        };
        let r = TxBuilder::new(
            box_selection,
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
        let inputs = vec![force_any_val::<ErgoBox>()];
        let outputs: Vec<ErgoBoxCandidate> = vec![];
        let r = TxBuilder::new(
            SimpleBoxSelector::new()
                .select(inputs, BoxValue::MIN, &[])
                .unwrap(),
            outputs,
            1,
            force_any_val::<BoxValue>(),
            force_any_val::<Address>(),
            BoxValue::SAFE_USER_MIN,
        );
        assert!(r.build().is_err());
    }

    #[test]
    fn test_burn_token() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 100.try_into().unwrap(),
        };
        let input_box = ErgoBox::new(
            10000000i64.try_into().unwrap(),
            force_any_val::<ErgoTree>(),
            vec![token_pair.clone()],
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        );
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let target_tokens = vec![Token {
            amount: 10.try_into().unwrap(),
            ..token_pair
        }];
        let box_selection = SimpleBoxSelector::new()
            .select(inputs, target_balance, target_tokens.as_slice())
            .unwrap();
        let box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        let out_box = box_builder.build().unwrap();
        let outputs = vec![out_box];
        let tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
            BoxValue::SAFE_USER_MIN,
        );
        let tx = tx_builder.build().unwrap();
        assert!(tx.output_candidates.get(0).unwrap().tokens().is_empty());
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

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

            let total_output_value: BoxValue = box_value::checked_sum(outputs.iter().map(|b| b.value)).unwrap()
                .checked_add(&miners_fee).unwrap();

            let mut tx_builder = TxBuilder::new(
                SimpleBoxSelector::new().select(inputs.clone(), total_output_value, &[]).unwrap(),
                outputs.clone(),
                1,
                miners_fee,
                change_address.clone(),
                min_change_value,
            );
            tx_builder.set_data_inputs(data_inputs.clone());
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
