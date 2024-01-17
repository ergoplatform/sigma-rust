//! Builder for an UnsignedTransaction

use ergotree_interpreter::eval::context::TxIoVec;
use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_ir::chain::token::TokenAmount;
use ergotree_ir::chain::token::TokenAmountError;
use ergotree_ir::ergo_tree::ErgoTree;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;

use bounded_vec::BoundedVecOutOfBounds;
use ergotree_interpreter::sigma_protocol;
use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use ergotree_ir::chain::address::Address;
use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::BoxId;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::token::Token;
use ergotree_ir::chain::token::TokenId;
use ergotree_ir::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializationError};
use thiserror::Error;

use crate::chain::contract::Contract;
use crate::chain::ergo_box::box_builder::{ErgoBoxCandidateBuilder, ErgoBoxCandidateBuilderError};
use crate::chain::transaction::unsigned::UnsignedTransaction;
use crate::chain::transaction::{DataInput, Input, Transaction, UnsignedInput};

use super::box_selector::subtract_tokens;
use super::box_selector::sum_tokens_from_boxes;
use super::box_selector::sum_value;
use super::box_selector::BoxSelection;
use super::box_selector::ErgoBoxAssets;
use super::box_selector::ErgoBoxId;
use super::miner_fee::MINERS_FEE_BASE16_BYTES;

/// Unsigned transaction builder
#[derive(Clone)]
pub struct TxBuilder<S: ErgoBoxAssets> {
    box_selection: BoxSelection<S>,
    data_inputs: Vec<DataInput>,
    output_candidates: Vec<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount: BoxValue,
    change_address: Address,
    context_extensions: HashMap<BoxId, ContextExtension>,
    token_burn_permit: Vec<Token>,
}

impl<S: ErgoBoxAssets + ErgoBoxId + Clone> TxBuilder<S> {
    /// Creates new TxBuilder
    /// `box_selection` - selected input boxes  (via [`super::box_selector::BoxSelector`])
    /// `output_candidates` - output boxes to be "created" in this transaction,
    /// `current_height` - chain height that will be used in additionally created boxes (change, miner's fee, etc.),
    /// `fee_amount` - miner's fee (higher values will speed up inclusion in blocks),
    /// `change_address` - change (inputs - outputs) will be sent to this address,
    /// will be given to miners,
    pub fn new(
        box_selection: BoxSelection<S>,
        output_candidates: Vec<ErgoBoxCandidate>,
        current_height: u32,
        fee_amount: BoxValue,
        change_address: Address,
    ) -> TxBuilder<S> {
        TxBuilder {
            box_selection,
            data_inputs: vec![],
            output_candidates,
            current_height,
            fee_amount,
            change_address,
            context_extensions: HashMap::new(),
            token_burn_permit: Vec::new(),
        }
    }

    /// Get inputs
    pub fn box_selection(&self) -> BoxSelection<S> {
        self.box_selection.clone()
    }

    /// Get data inputs
    pub fn data_inputs(&self) -> Vec<DataInput> {
        self.data_inputs.clone()
    }

    /// Get outputs
    pub fn output_candidates(&self) -> Vec<ErgoBoxCandidate> {
        self.output_candidates.clone()
    }

    /// Get current height
    pub fn current_height(&self) -> u32 {
        self.current_height
    }

    /// Get fee amount
    pub fn fee_amount(&self) -> BoxValue {
        self.fee_amount
    }

    /// Get change
    pub fn change_address(&self) -> Address {
        self.change_address.clone()
    }

    /// Set transaction's data inputs
    pub fn set_data_inputs(&mut self, data_inputs: Vec<DataInput>) {
        self.data_inputs = data_inputs;
    }

    /// Set context extension for a given input
    pub fn set_context_extension(&mut self, box_id: BoxId, context_extension: ContextExtension) {
        self.context_extensions.insert(box_id, context_extension);
    }

    /// Estimated serialized transaction size in bytes after signing (assuming P2PK box spending)
    pub fn estimate_tx_size_bytes(&self) -> Result<usize, TxBuilderError> {
        let tx = self.build_tx()?;
        let inputs = tx.inputs.mapped(|ui| {
            // mock proof of the size of ProveDlog's proof (P2PK box spending)
            // as it's the most often used proof
            let proof = ProofBytes::Some(vec![0u8, sigma_protocol::SOUNDNESS_BYTES as u8]);
            Input::new(
                ui.box_id,
                crate::chain::transaction::input::prover_result::ProverResult {
                    proof,
                    extension: ui.extension,
                },
            )
        });
        let signed_tx_mock = Transaction::new(inputs, tx.data_inputs, tx.output_candidates)?;
        Ok(signed_tx_mock.sigma_serialize_bytes()?.len())
    }

    /// Permits the burn of the given token amount, i.e. allows this token amount to be omitted in the outputs
    pub fn set_token_burn_permit(&mut self, tokens: Vec<Token>) {
        self.token_burn_permit = tokens;
    }

    fn build_tx(&self) -> Result<UnsignedTransaction, TxBuilderError> {
        if self.box_selection.boxes.is_empty() {
            return Err(TxBuilderError::InvalidArgs("inputs are empty".to_string()));
        }
        if self.output_candidates.is_empty() {
            return Err(TxBuilderError::InvalidArgs("outputs are empty".to_string()));
        }
        if self.box_selection.boxes.len() > u16::MAX as usize {
            return Err(TxBuilderError::InvalidArgs("too many inputs".to_string()));
        }
        if self
            .box_selection
            .boxes
            .clone()
            .into_iter()
            .map(|b| b.box_id())
            .collect::<HashSet<BoxId>>()
            .len()
            != self.box_selection.boxes.len()
        {
            return Err(TxBuilderError::InvalidArgs(
                "duplicate inputs found".to_string(),
            ));
        }
        if self.data_inputs.len() > u16::MAX as usize {
            return Err(TxBuilderError::InvalidArgs(
                "too many data inputs".to_string(),
            ));
        }

        let mut output_candidates = self.output_candidates.clone();
        let change_address_ergo_tree = Contract::pay_to_address(&self.change_address)?.ergo_tree();
        let change_boxes: Result<Vec<ErgoBoxCandidate>, ErgoBoxCandidateBuilderError> = self
            .box_selection
            .change_boxes
            .iter()
            .map(|b| {
                let mut candidate = ErgoBoxCandidateBuilder::new(
                    b.value,
                    change_address_ergo_tree.clone(),
                    self.current_height,
                );
                for token in b.tokens().into_iter().flatten() {
                    candidate.add_token(token.clone());
                }
                candidate.build()
            })
            .collect();
        output_candidates.append(&mut change_boxes?);

        // add miner's fee
        let miner_fee_box = new_miner_fee_box(self.fee_amount, self.current_height)?;
        output_candidates.push(miner_fee_box);
        if output_candidates.len() > Transaction::MAX_OUTPUTS_COUNT {
            return Err(TxBuilderError::InvalidArgs("too many outputs".to_string()));
        }
        // check input's coins preservation
        let total_input_value = sum_value(self.box_selection.boxes.as_slice());
        let total_output_value = sum_value(output_candidates.as_slice());
        #[allow(clippy::comparison_chain)]
        if total_output_value > total_input_value {
            return Err(TxBuilderError::NotEnoughCoinsInInputs(
                total_output_value - total_input_value,
            ));
        } else if total_output_value < total_input_value {
            return Err(TxBuilderError::NotEnoughCoinsInOutputs(
                total_input_value - total_output_value,
            ));
        }

        // check that inputs have enough tokens
        let input_tokens = sum_tokens_from_boxes(self.box_selection.boxes.as_slice())
            .map_err(TxBuilderError::TooManyTokensInInputBoxes)?;
        let output_tokens = sum_tokens_from_boxes(output_candidates.as_slice())
            .map_err(TxBuilderError::TooManyTokensInOutputCandidates)?;
        let first_input_box_id: TokenId = self.box_selection.boxes.first().box_id().into();
        let output_tokens_len = output_tokens.len();
        let output_tokens_without_minted: HashMap<TokenId, TokenAmount> = output_tokens
            .into_iter()
            .filter(|(id, _)| id != &first_input_box_id)
            .collect();
        if output_tokens_len - output_tokens_without_minted.len() > 1 {
            return Err(TxBuilderError::InvalidArgs(
                "cannot mint more than one token".to_string(),
            ));
        }
        output_tokens_without_minted
            .iter()
            .try_for_each(|(id, amt)| match input_tokens.get(id).cloned() {
                Some(input_token_amount) if input_token_amount >= *amt => Ok(()),
                _ => Err(TxBuilderError::NotEnoughTokens(vec![(*id, *amt).into()])),
            })?;

        // check token burn
        let burned_tokens = subtract_tokens(&input_tokens, &output_tokens_without_minted)
            .map_err(TxBuilderError::TokensInOutputsExceedInputs)?;
        let token_burn_permits = vec_tokens_to_map(self.token_burn_permit.clone())
            .map_err(TxBuilderError::TooManyTokensInBurnPermit)?;
        check_enough_token_burn_permit(&burned_tokens, &token_burn_permits)?;
        check_unused_token_burn_permit(&burned_tokens, &token_burn_permits)?;

        let unsigned_inputs = self.box_selection.boxes.clone().mapped(|b| {
            let ctx_ext = self
                .context_extensions
                .get(&b.box_id())
                .cloned()
                .unwrap_or_else(ContextExtension::empty);
            UnsignedInput::new(b.box_id(), ctx_ext)
        });
        Ok(UnsignedTransaction::new(
            unsigned_inputs,
            TxIoVec::opt_empty_vec(self.data_inputs.clone())?,
            output_candidates.try_into()?,
        )?)
    }

    /// Build the unsigned transaction
    pub fn build(self) -> Result<UnsignedTransaction, TxBuilderError> {
        self.build_tx()
    }
}

/// Suggested transaction fee (1100000 nanoERGs, semi-default value used across wallets and dApps as of Oct 2020)
#[allow(non_snake_case, clippy::unwrap_used)]
pub fn SUGGESTED_TX_FEE() -> BoxValue {
    BoxValue::new(1100000u64).unwrap()
}

/// Create a box with miner's contract and a given value
#[allow(clippy::unwrap_used)]
pub fn new_miner_fee_box(
    fee_amount: BoxValue,
    creation_height: u32,
) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
    let ergo_tree =
        ErgoTree::sigma_parse_bytes(base16::decode(MINERS_FEE_BASE16_BYTES).unwrap().as_slice())
            .unwrap();
    ErgoBoxCandidateBuilder::new(fee_amount, ergo_tree, creation_height).build()
}

/// Errors of TxBuilder
#[allow(missing_docs)]
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum TxBuilderError {
    #[error("SigmaParsingError: {0}")]
    ParsingError(#[from] SigmaParsingError),
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("ErgoBoxCandidateBuilder error: {0}")]
    ErgoBoxCandidateBuilderError(#[from] ErgoBoxCandidateBuilderError),
    #[error("Not enougn tokens: {0:?}")]
    NotEnoughTokens(Vec<Token>),
    #[error("Not enough coins({0} nanoERGs are missing)")]
    NotEnoughCoinsInInputs(u64),
    #[error("Transaction serialization failed: {0}")]
    SerializationError(#[from] SigmaSerializationError),
    #[error("Invalid tx inputs count: {0}")]
    InvalidInputsCount(#[from] BoundedVecOutOfBounds),
    #[error("Empty input box")]
    EmptyInputBoxSelection,
    #[error("Token burn permit exceeded. Permitted limit: {permit:?}, trying to burn: {try_to_burn:?}. Revisit the input to `set_token_burn_permit()` to increase the limit")]
    TokenBurnPermitExceeded { permit: Token, try_to_burn: Token },
    #[error("Token burn permit is missing. Trying to burn: {try_to_burn:?}. Call `set_token_burn_permit()` to set the limit")]
    TokenBurnPermitMissing { try_to_burn: Token },
    #[error("Unused token burn permit: token id {token_id:?}, amount {amount:?}")]
    TokenBurnPermitUnused { token_id: TokenId, amount: u64 },
    #[error("Too many tokens in burn permit: {0}")]
    TooManyTokensInBurnPermit(TokenAmountError),
    #[error("Too many tokens in input boxes: {0}")]
    TooManyTokensInInputBoxes(TokenAmountError),
    #[error("Too many tokens in output candidate boxes: {0}")]
    TooManyTokensInOutputCandidates(TokenAmountError),
    #[error("Tokens in output candidate exceed tokens in input boxes: {0}")]
    TokensInOutputsExceedInputs(TokenAmountError),
    #[error("Coins in outputs are less than coins in inputs for {0} nanoERGs")]
    NotEnoughCoinsInOutputs(u64),
}

/// Sums up the tokens into a hash map
pub(crate) fn vec_tokens_to_map(
    tokens: Vec<Token>,
) -> Result<HashMap<TokenId, TokenAmount>, TokenAmountError> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    tokens.iter().try_for_each(|b| {
        if let Some(amt) = res.get_mut(&b.token_id) {
            *amt = amt.checked_add(&b.amount)?;
        } else {
            res.insert(b.token_id, b.amount);
        }
        Ok(())
    })?;
    Ok(res)
}

fn check_enough_token_burn_permit(
    burned_tokens: &HashMap<TokenId, TokenAmount>,
    permits: &HashMap<TokenId, TokenAmount>,
) -> Result<(), TxBuilderError> {
    for (burn_token_id, burn_amt) in burned_tokens {
        if let Some(burn_amt_permit) = permits.get(burn_token_id) {
            if burn_amt > burn_amt_permit {
                return Err(TxBuilderError::TokenBurnPermitExceeded {
                    permit: (*burn_token_id, *burn_amt_permit).into(),
                    try_to_burn: (*burn_token_id, *burn_amt).into(),
                });
            }
        } else {
            return Err(TxBuilderError::TokenBurnPermitMissing {
                try_to_burn: (*burn_token_id, *burn_amt).into(),
            });
        }
    }
    Ok(())
}

fn check_unused_token_burn_permit(
    burned_tokens: &HashMap<TokenId, TokenAmount>,
    permits: &HashMap<TokenId, TokenAmount>,
) -> Result<(), TxBuilderError> {
    for (permit_token_id, permit_amt) in permits {
        if let Some(burn_amt) = burned_tokens.get(permit_token_id) {
            if burn_amt < permit_amt {
                return Err(TxBuilderError::TokenBurnPermitUnused {
                    token_id: *permit_token_id,
                    amount: *permit_amt.as_u64() - *burn_amt.as_u64(),
                });
            }
        } else {
            return Err(TxBuilderError::TokenBurnPermitUnused {
                token_id: *permit_token_id,
                amount: *permit_amt.as_u64(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {

    use std::convert::TryInto;

    use ergotree_ir::chain::ergo_box::arbitrary::ArbBoxParameters;
    use ergotree_ir::chain::ergo_box::box_value::checked_sum;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
    use ergotree_ir::chain::token::arbitrary::ArbTokenIdParam;
    use ergotree_ir::chain::token::TokenAmount;
    use ergotree_ir::chain::tx_id::TxId;
    use ergotree_ir::ergo_tree::ErgoTree;
    use proptest::{collection::vec, prelude::*};
    use sigma_test_util::force_any_val;
    use sigma_test_util::force_any_val_with;

    use crate::wallet::box_selector::{BoxSelector, SimpleBoxSelector};

    use super::*;

    #[test]
    fn test_duplicate_inputs() {
        let input_box = force_any_val::<ErgoBox>();
        let box_selection: BoxSelection<ErgoBox> = BoxSelection {
            boxes: vec![input_box.clone(), input_box].try_into().unwrap(),
            change_boxes: vec![],
        };
        let r = TxBuilder::new(
            box_selection,
            vec![force_any_val::<ErgoBoxCandidate>()],
            1,
            force_any_val::<BoxValue>(),
            force_any_val::<Address>(),
        );
        assert!(matches!(r.build(), Err(TxBuilderError::InvalidArgs(_))));
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
        );
        assert!(matches!(r.build(), Err(TxBuilderError::InvalidArgs(_))));
    }

    #[test]
    fn test_burn_token_wo_permit() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 100.try_into().unwrap(),
        };
        let input_box = ErgoBox::new(
            10000000i64.try_into().unwrap(),
            force_any_val::<ErgoTree>(),
            vec![token_pair.clone()].try_into().ok(),
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        )
        .unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let target_token = Token {
            amount: 10.try_into().unwrap(),
            ..token_pair
        };
        let target_tokens = vec![target_token.clone()];
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
        );
        let res = tx_builder.build();
        assert_eq!(
            res,
            Err(TxBuilderError::TokenBurnPermitMissing {
                try_to_burn: target_token
            })
        );
    }

    #[test]
    fn test_burn_token_w_permit_too_low() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 100.try_into().unwrap(),
        };
        let input_box = ErgoBox::new(
            10000000i64.try_into().unwrap(),
            force_any_val::<ErgoTree>(),
            vec![token_pair.clone()].try_into().ok(),
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        )
        .unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let token_to_burn = Token {
            amount: 10.try_into().unwrap(),
            ..token_pair
        };
        let target_tokens = vec![token_to_burn.clone()];
        let box_selection = SimpleBoxSelector::new()
            .select(inputs, target_balance, target_tokens.as_slice())
            .unwrap();
        let box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        let out_box = box_builder.build().unwrap();
        let outputs = vec![out_box];
        let mut tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        let token_burn_permit = Token {
            amount: 5.try_into().unwrap(),
            ..token_pair
        };
        tx_builder.set_token_burn_permit(vec![token_burn_permit.clone()]);
        let res = tx_builder.build();
        assert_eq!(
            res,
            Err(TxBuilderError::TokenBurnPermitExceeded {
                try_to_burn: token_to_burn,
                permit: token_burn_permit,
            })
        );
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
            vec![token_pair.clone()].try_into().ok(),
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        )
        .unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let token_to_burn = Token {
            amount: 10.try_into().unwrap(),
            ..token_pair
        };
        let target_tokens = vec![token_to_burn.clone()];
        let box_selection = SimpleBoxSelector::new()
            .select(inputs, target_balance, target_tokens.as_slice())
            .unwrap();
        let box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        let out_box = box_builder.build().unwrap();
        let outputs = vec![out_box];
        let mut tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        tx_builder.set_token_burn_permit(vec![token_to_burn]);
        let _ = tx_builder.build().unwrap();
    }

    #[test]
    fn test_token_burn_permit_wo_burn() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 100.try_into().unwrap(),
        };
        let input_box = ErgoBox::new(
            10000000i64.try_into().unwrap(),
            force_any_val::<ErgoTree>(),
            vec![token_pair.clone()].try_into().ok(),
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        )
        .unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let token_to_burn = Token {
            amount: 10.try_into().unwrap(),
            ..token_pair
        };
        let box_selection = SimpleBoxSelector::new()
            .select(inputs, target_balance, &Vec::new())
            .unwrap();
        let box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        let out_box = box_builder.build().unwrap();
        let outputs = vec![out_box];
        let mut tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        tx_builder.set_token_burn_permit(vec![token_to_burn.clone()]);
        let res = tx_builder.build();
        assert_eq!(
            res,
            Err(TxBuilderError::TokenBurnPermitUnused {
                token_id: token_to_burn.token_id,
                amount: *token_to_burn.amount.as_u64(),
            })
        );
    }

    #[test]
    fn test_mint_token() {
        let input_box = ErgoBox::new(
            100000000i64.try_into().unwrap(),
            force_any_val::<ErgoTree>(),
            None,
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        )
        .unwrap();
        let token_pair = Token {
            token_id: TokenId::from(input_box.box_id()),
            amount: 1.try_into().unwrap(),
        };
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let token_name = "TKN".to_string();
        let token_desc = "token desc".to_string();
        let token_num_dec = 2;
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        box_builder.mint_token(token_pair.clone(), token_name, token_desc, token_num_dec);
        let out_box = box_builder.build().unwrap();

        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let box_selection = SimpleBoxSelector::new()
            .select(inputs, target_balance, vec![].as_slice())
            .unwrap();
        let outputs = vec![out_box];
        let tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        let tx = tx_builder.build().unwrap();
        assert_eq!(
            tx.output_candidates
                .get(0)
                .unwrap()
                .tokens()
                .unwrap()
                .first()
                .token_id,
            token_pair.token_id,
            "expected minted token in the first output box"
        );
    }

    #[test]
    fn test_tokens_balance_error() {
        let input_box = force_any_val_with::<ErgoBox>(
            ArbBoxParameters { value_range: (BoxValue::MIN_RAW * 5000..BoxValue::MIN_RAW * 10000).into(), ..Default::default() },
        );
        let token_pair = Token {
            token_id: force_any_val_with::<TokenId>(ArbTokenIdParam::Arbitrary),
            amount: force_any_val::<TokenAmount>(),
        };
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        // try to spend a token that is not in inputs
        box_builder.add_token(token_pair.clone());
        let out_box = box_builder.build().unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let target_balance = out_box_value.checked_add(&tx_fee).unwrap();
        let box_selection = SimpleBoxSelector::new()
            .select(inputs, target_balance, vec![].as_slice())
            .unwrap();
        let outputs = vec![out_box];
        let tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        assert_eq!(
            tx_builder.build(),
            Err(TxBuilderError::NotEnoughTokens(vec![token_pair])),
        );
    }

    #[test]
    fn test_balance_error_not_enough_inputs() {
        let input_box = force_any_val_with::<ErgoBox>(
            ArbBoxParameters { value_range: (BoxValue::MIN_RAW * 5000..BoxValue::MIN_RAW * 10000).into(), ..Default::default() },
        );
        // tx fee on top of this leads to overspending
        let out_box_value = input_box.value();
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        input_box.tokens.iter().for_each(|tokens| {
            tokens.iter().for_each(|t| {
                box_builder.add_token(t.clone());
            })
        });
        let out_box = box_builder.build().unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let box_selection = BoxSelection {
            boxes: inputs.try_into().unwrap(),
            change_boxes: vec![],
        };
        let outputs = vec![out_box];
        let tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        assert_eq!(
            tx_builder.build(),
            Err(TxBuilderError::NotEnoughCoinsInInputs(
                *BoxValue::SAFE_USER_MIN.as_u64()
            )),
        );
    }

    #[test]
    fn test_balance_error_not_enough_outputs() {
        let input_box = force_any_val_with::<ErgoBox>(
            ArbBoxParameters { value_range: (BoxValue::MIN_RAW * 5000..BoxValue::MIN_RAW * 10000).into(), ..Default::default() },
        );
        // spend not all inputs
        let out_box_value = input_box
            .value()
            // goes to tx fee
            .checked_sub(&BoxValue::SAFE_USER_MIN)
            .unwrap()
            .checked_sub(&BoxValue::SAFE_USER_MIN)
            .unwrap();
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        input_box.tokens.iter().for_each(|tokens| {
            tokens.iter().for_each(|t| {
                box_builder.add_token(t.clone());
            })
        });
        let out_box = box_builder.build().unwrap();
        let inputs: Vec<ErgoBox> = vec![input_box];
        let tx_fee = BoxValue::SAFE_USER_MIN;
        let box_selection = BoxSelection {
            boxes: inputs.try_into().unwrap(),
            change_boxes: vec![],
        };
        let outputs = vec![out_box];
        let tx_builder = TxBuilder::new(
            box_selection,
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        assert_eq!(
            tx_builder.build(),
            Err(TxBuilderError::NotEnoughCoinsInOutputs(
                *BoxValue::SAFE_USER_MIN.as_u64()
            )),
        );
    }

    #[test]
    fn test_est_tx_size() {
        let input = ErgoBox::new(
            10000000i64.try_into().unwrap(),
            force_any_val::<ErgoTree>(),
            None,
            NonMandatoryRegisters::empty(),
            1,
            force_any_val::<TxId>(),
            0,
        )
        .unwrap();
        let tx_fee = super::SUGGESTED_TX_FEE();
        let out_box_value = input.value.checked_sub(&tx_fee).unwrap();
        let box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        let out_box = box_builder.build().unwrap();
        let outputs = vec![out_box];
        let tx_builder = TxBuilder::new(
            BoxSelection {
                boxes: vec![input].try_into().unwrap(),
                change_boxes: vec![],
            },
            outputs,
            0,
            tx_fee,
            force_any_val::<Address>(),
        );
        assert!(tx_builder.estimate_tx_size_bytes().unwrap() > 0);
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_build_tx(inputs in vec(any_with::<ErgoBox>(ArbBoxParameters { value_range: (BoxValue::MIN_RAW * 5000..BoxValue::MIN_RAW * 10000).into(), ..Default::default() }), 1..10),
                         outputs in vec(any_with::<ErgoBoxCandidate>(ArbBoxParameters { value_range: (BoxValue::MIN_RAW * 5000..BoxValue::MIN_RAW * 10000).into(), ..Default::default() }), 1..2),
                         change_address in any::<Address>(),
                         miners_fee in any_with::<BoxValue>((BoxValue::MIN_RAW * 100..BoxValue::MIN_RAW * 200).into()),
                         data_inputs in vec(any::<DataInput>(), 0..2),
                         ctx_ext in any::<ContextExtension>()) {
            prop_assume!(sum_tokens_from_boxes(outputs.as_slice()).unwrap().is_empty());
            let all_outputs = checked_sum(outputs.iter().map(|b| b.value)).unwrap()
                .checked_add(&miners_fee)
                .unwrap();
            let all_inputs = checked_sum(inputs.iter().map(|b| b.value)).unwrap();
            prop_assume!(all_outputs < all_inputs);
            let total_output_value: BoxValue = checked_sum(outputs.iter().map(|b| b.value))
                .unwrap()
                .checked_add(&miners_fee).unwrap();
            let selection = SimpleBoxSelector::new().select(inputs.clone(), total_output_value, &[]).unwrap();
            let mut tx_builder = TxBuilder::new(
                selection.clone(),
                outputs.clone(),
                1,
                miners_fee,
                change_address.clone(),
            );
            tx_builder.set_data_inputs(data_inputs.clone());
            tx_builder.set_context_extension(selection.boxes.first().box_id(), ctx_ext.clone());
            let tx = tx_builder.build().unwrap();
            prop_assert!(outputs.into_iter().all(|i| tx.output_candidates.iter().any(|o| *o == i)),
                         "tx.output_candidates is missing some outputs");
            let tx_all_inputs_vals = tx.inputs.iter()
                .map(|i| inputs.iter()
                    .find(|ib| ib.box_id() == i.box_id).unwrap().value);
            let tx_all_inputs_sum = checked_sum(tx_all_inputs_vals).unwrap();
            let expected_change = tx_all_inputs_sum.checked_sub(&all_outputs).unwrap();
            prop_assert!(tx.output_candidates.iter().any(|b| {
                b.value == expected_change && b.ergo_tree == change_address.script().unwrap()
            }), "box with change {:?} is not found in outputs: {:?}", expected_change, tx.output_candidates);
            prop_assert!(tx.output_candidates.iter().any(|b| {
                b.value == miners_fee
            }), "box with miner's fee {:?} is not found in outputs: {:?}", miners_fee, tx.output_candidates);
            prop_assert_eq!(tx.data_inputs.map(|i| i.as_vec().clone()).unwrap_or_default(), data_inputs, "unexpected data inputs");
            prop_assert_eq!(&tx.inputs.first().extension, &ctx_ext);
        }
    }
}
