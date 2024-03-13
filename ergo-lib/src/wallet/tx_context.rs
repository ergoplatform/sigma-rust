//! Transaction context

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::ergo_transaction::{ErgoTransaction, TxValidationError};
use crate::chain::transaction::{verify_tx_input_proof, Transaction, TransactionError};
use crate::ergotree_ir::chain::ergo_box::BoxId;
use ergotree_interpreter::eval::context::TxIoVec;
use ergotree_interpreter::sigma_protocol::verifier::VerificationResult;
use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::{BoxTokens, ErgoBox};
use ergotree_ir::chain::token::{TokenAmount, TokenId};
use ergotree_ir::serialization::SigmaSerializable;
use thiserror::Error;

/// Transaction and an additional info required for signing or verification
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TransactionContext<T: ErgoTransaction> {
    /// Unsigned transaction to sign
    pub spending_tx: T,
    /// Boxes corresponding to [`crate::chain::transaction::unsigned::UnsignedTransaction::inputs`]
    boxes_to_spend: TxIoVec<ErgoBox>,
    /// Boxes corresponding to [`crate::chain::transaction::unsigned::UnsignedTransaction::data_inputs`]
    pub(crate) data_boxes: Option<TxIoVec<ErgoBox>>,
    /// Stores the location of each BoxId in [`Self::boxes_to_spend`]
    box_index: HashMap<BoxId, u16>,
}

impl<T: ErgoTransaction> TransactionContext<T> {
    /// New TransactionContext
    pub fn new(
        spending_tx: T,
        boxes_to_spend: Vec<ErgoBox>,
        data_boxes: Vec<ErgoBox>,
    ) -> Result<Self, TransactionContextError> {
        let boxes_to_spend = TxIoVec::from_vec(boxes_to_spend).map_err(|e| match e {
            bounded_vec::BoundedVecOutOfBounds::LowerBoundError { .. } => {
                TransactionContextError::NoInputBoxes
            }
            bounded_vec::BoundedVecOutOfBounds::UpperBoundError { got, .. } => {
                TransactionContextError::TooManyInputBoxes(got)
            }
        })?;
        let data_boxes_len = data_boxes.len();
        let data_boxes = if !data_boxes.is_empty() {
            Some(
                TxIoVec::from_vec(data_boxes)
                    .map_err(|_| TransactionContextError::TooManyDataInputBoxes(data_boxes_len))?,
            )
        } else {
            None
        };

        let box_index: HashMap<BoxId, u16> = boxes_to_spend
            .iter()
            .enumerate()
            .map(|(i, b)| (b.box_id(), i as u16))
            .collect();
        for (i, unsigned_input) in spending_tx.inputs_ids().iter().enumerate() {
            if !box_index.contains_key(&unsigned_input) {
                return Err(TransactionContextError::InputBoxNotFound(i));
            }
        }

        if let Some(data_inputs) = spending_tx.data_inputs().as_ref() {
            if let Some(data_boxes) = data_boxes.as_ref() {
                let data_box_index: HashMap<BoxId, u16> = data_boxes
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (b.box_id(), i as u16))
                    .collect();
                for (i, data_input) in data_inputs.iter().enumerate() {
                    if !data_box_index.contains_key(&data_input.box_id) {
                        return Err(TransactionContextError::DataInputBoxNotFound(i));
                    }
                }
            } else {
                return Err(TransactionContextError::DataInputBoxNotFound(0));
            }
        }
        Ok(TransactionContext {
            spending_tx,
            boxes_to_spend,
            data_boxes,
            box_index,
        })
    }

    /// Returns box with given id, if it exists.
    pub fn get_input_box(&self, box_id: &BoxId) -> Option<ErgoBox> {
        self.box_index
            .get(box_id)
            .and_then(|&idx| self.boxes_to_spend.get(idx as usize))
            .cloned()
    }
}

impl TransactionContext<Transaction> {
    /// Verify transaction using blockchain parameters
    // TODO: costing, storage rent, re-emission
    // This is based on validateStateful() in Ergo: https://github.com/ergoplatform/ergo/blob/48239ef98ced06617dc21a0eee5670235e362933/ergo-core/src/main/scala/org/ergoplatform/modifiers/mempool/ErgoTransaction.scala#L357
    pub fn validate(&self, state_context: &ErgoStateContext) -> Result<(), TxValidationError> {
        // Check that input sum does not overflow
        let input_sum = BoxValue::new(
            self.boxes_to_spend
                .iter()
                .map(|b| b.value.as_u64())
                .sum::<u64>(),
        )
        .map_err(|_| TxValidationError::InputSumOverflow)?;
        // Check that output sum does not overflow and is equal to ERG amount in inputs
        let output_sum = self
            .spending_tx
            .outputs
            .iter()
            .map(|b| b.value.as_u64())
            .sum();
        if *input_sum.as_u64() != output_sum {
            return Err(TxValidationError::ErgPreservationError(
                *input_sum.as_u64(),
                output_sum,
            ));
        }

        // Monotonic Box creation happens after v3
        let max_creation_height = if state_context.pre_header.version <= 2 {
            0
        } else {
            #[allow(clippy::unwrap_used)] // Unwrap is valid here since inputs can not be empty
            self.boxes_to_spend
                .iter()
                .map(|b| b.creation_height)
                .max()
                .unwrap()
        };
        // Check that outputs are not dust and aren't created in future
        for output in &self.spending_tx.outputs {
            verify_output(state_context, output, max_creation_height)?;
        }

        let in_assets = extract_assets(self.boxes_to_spend.iter().map(|b| &b.tokens))?;
        let out_assets = extract_assets(self.spending_tx.outputs.iter().map(|b| &b.tokens))?;
        verify_assets(
            self.spending_tx.inputs_ids().as_slice(),
            in_assets,
            out_assets,
        )?;
        // Verify input proofs. This is usually the most expensive check so it's done last
        for input_idx in 0..self.spending_tx.inputs.len() {
            if let res @ VerificationResult { result: false, .. } =
                verify_tx_input_proof(self, state_context, input_idx)?
            {
                return Err(TxValidationError::ReducedToFalse(input_idx, res));
            }
        }
        Ok(())
    }
}

// TODO: check that box creation height does not exceed height in preheader
fn verify_output(
    state_context: &ErgoStateContext,
    output: &ErgoBox,
    max_creation_height: u32,
) -> Result<(), TxValidationError> {
    let box_size = output.sigma_serialize_bytes()?.len() as u64;
    let script_size = output.script_bytes()?.len();
    let block_version = state_context.pre_header.version;
    // Check that output is not dust
    let minimum_value = box_size * state_context.parameters.min_value_per_byte() as u64;
    if *output.value.as_u64() < minimum_value {
        return Err(TxValidationError::DustOutput(
            output.box_id(),
            output.value,
            minimum_value,
        ));
    }
    // Check that height does not exceed maximum height. Note that heights can be potentially negative in V1
    if output.creation_height as i32 > state_context.pre_header.height as i32 {
        return Err(TxValidationError::InvalidHeightError(
            output.creation_height,
        ));
    }
    if output.creation_height < max_creation_height {
        return Err(TxValidationError::MonotonicHeightError(
            output.creation_height,
            max_creation_height,
        ));
    }
    // Negative output heights were allowed in V1. sigma-rust always stores heights as unsigned integers
    if block_version != 1 && output.creation_height & (1 << 31) != 0 {
        return Err(TxValidationError::NegativeHeight);
    }
    if box_size as usize > ErgoBox::MAX_BOX_SIZE {
        return Err(TxValidationError::BoxSizeExceeded(box_size as usize));
    }
    if script_size > ErgoBox::MAX_SCRIPT_SIZE {
        return Err(TxValidationError::ScriptSizeExceeded(script_size));
    }
    Ok(())
}

// Extract all of the assets in a collection of boxes for transaction validation
fn extract_assets<'a, I: Iterator<Item = &'a Option<BoxTokens>>>(
    mut boxes: I,
) -> Result<HashMap<TokenId, TokenAmount>, TxValidationError> {
    boxes.try_fold(
        HashMap::new(),
        |mut asset_map: HashMap<TokenId, TokenAmount>, tokens| {
            tokens
                .as_ref()
                .into_iter()
                .flatten()
                .try_for_each(|token| {
                    match asset_map.entry(token.token_id) {
                        Entry::Occupied(mut occ) => {
                            *occ.get_mut() = occ.get().checked_add(&token.amount)?;
                        }
                        Entry::Vacant(vac) => {
                            vac.insert(token.amount);
                        }
                    }
                    Ok::<(), TxValidationError>(())
                })?;
            Ok(asset_map)
        },
    )
}

fn verify_assets(
    inputs: &[BoxId],
    in_assets: HashMap<TokenId, TokenAmount>,
    out_assets: HashMap<TokenId, TokenAmount>,
) -> Result<(), TxValidationError> {
    // If this transaction mints a new token, it's token ID must be the ID of the first box being spent
    let new_token_id: TokenId = inputs[0].into();
    for (&out_token_id, &out_amount) in &out_assets {
        if let Some(&in_amount) = in_assets.get(&out_token_id) {
            // Check that Transaction is not creating tokens out of thin air
            if in_amount < out_amount {
                return Err(TxValidationError::TokenPreservationError {
                    token_id: out_token_id,
                    in_amount: in_amount.into(),
                    out_amount: out_amount.into(),
                    new_token_id,
                });
            }
        } else if out_token_id != new_token_id {
            //minting a new token. Token amount checks are handled by the TokenAmount newtype and not needed here
            return Err(TxValidationError::TokenPreservationError {
                token_id: out_token_id,
                in_amount: 0,
                out_amount: out_amount.into(),
                new_token_id,
            });
        }
    }
    Ok(())
}

/// Transaction context errors
#[derive(Error, Debug)]
pub enum TransactionContextError {
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(#[from] TransactionError),
    /// No input boxes (boxes_to_spend is empty)
    #[error("No input boxes")]
    NoInputBoxes,
    /// Too many input boxes
    #[error("Too many input boxes: {0}")]
    TooManyInputBoxes(usize),
    /// Input box not found
    #[error("Input box not found: {0}")]
    InputBoxNotFound(usize),
    /// Too many data input boxes
    #[error("Too many data input boxes: {0}")]
    TooManyDataInputBoxes(usize),
    /// Data input box not found
    #[error("Data input box not found: {0}")]
    DataInputBoxNotFound(usize),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod test {
    use std::collections::HashMap;

    use ergotree_interpreter::eval::context::TxIoVec;
    use ergotree_interpreter::sigma_protocol::prover::{ContextExtension, ProofBytes};
    use ergotree_ir::chain::ergo_box::arbitrary::ArbBoxParameters;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::{
        BoxTokens, ErgoBox, ErgoBoxCandidate, NonMandatoryRegisters,
    };
    use ergotree_ir::chain::token::arbitrary::ArbTokenIdParam;
    use ergotree_ir::chain::token::{Token, TokenAmount, TokenId};
    use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeHeader};
    use ergotree_ir::mir::constant::{Constant, Literal};
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use proptest::strategy::Strategy;
    use proptest::test_runner::TestRng;
    use sigma_test_util::{force_any_val, force_any_val_with};

    use crate::chain::ergo_state_context::ErgoStateContext;
    use crate::chain::parameters::Parameters;
    use crate::chain::transaction::ergo_transaction::{ErgoTransaction, TxValidationError};
    use crate::chain::transaction::prover_result::ProverResult;
    use crate::chain::transaction::unsigned::UnsignedTransaction;
    use crate::chain::transaction::{Input, Transaction, UnsignedInput};
    use crate::wallet::Wallet;

    use super::TransactionContext;

    // Disperse token_count tokens across inputs
    fn disperse_tokens(inputs: u16, token_count: u8) -> Vec<Option<BoxTokens>> {
        let mut token_distribution = vec![vec![]; inputs as usize];
        for i in 0..token_count {
            let token = force_any_val_with::<Token>(ArbTokenIdParam::Arbitrary);
            token_distribution[(i as usize) % inputs as usize].push(token);
        }
        token_distribution
            .into_iter()
            .map(BoxTokens::from_vec)
            .map(Result::ok)
            .collect()
    }
    fn gen_boxes(
        min_tokens: u8,
        max_tokens: u8,
        min_inputs: u16,
        max_inputs: u16,
        ergotree_gen: impl Strategy<Value = ErgoTree>,
        height_gen: Option<BoxedStrategy<u32>>,
    ) -> impl Strategy<Value = Vec<ErgoBox>> {
        (
            min_inputs..=max_inputs,
            min_tokens..=max_tokens,
            ergotree_gen,
            height_gen.clone().unwrap_or_else(|| Just(0).boxed()),
        )
            .prop_flat_map(
                |(input_count, assets_count, proposition, creation_height)| {
                    let tokens = disperse_tokens(input_count, assets_count);
                    tokens
                        .into_iter()
                        .map(move |tokens| {
                            let box_params = ArbBoxParameters {
                                value_range: (1000000..100000000).into(),
                                ergo_tree: Just(proposition.clone()).boxed(),
                                creation_height: Just(creation_height).boxed(),
                                tokens: Just(tokens).boxed(),
                                ..Default::default()
                            };
                            ErgoBox::arbitrary_with(box_params)
                        })
                        .collect::<Vec<_>>()
                },
            )
    }
    fn valid_unsigned_transaction_from_boxes(
        mut rng: TestRng,
        boxes: &[ErgoBox],
        issue_new_token: bool,
        output_prop: ErgoTree,
        _data_boxes: &[ErgoBox],
    ) -> UnsignedTransaction {
        let input_sum = boxes.iter().map(|b| *b.value.as_u64()).sum::<u64>();
        assert!(input_sum > *BoxValue::SAFE_USER_MIN.as_u64());

        let mut assets_map: HashMap<TokenId, TokenAmount> = boxes
            .iter()
            .flat_map(|b| b.tokens.clone().into_iter().flatten())
            .map(|token| (token.token_id, token.amount))
            .collect();
        if issue_new_token {
            assets_map.insert(
                boxes[0].box_id().into(),
                rng.gen_range(1..=i64::MAX as u64).try_into().unwrap(),
            );
        }
        let max_outputs = std::cmp::min(
            i16::MAX as u16,
            (input_sum / BoxValue::SAFE_USER_MIN.as_u64()) as u16,
        );
        let outputs = std::cmp::min(
            max_outputs,
            std::cmp::max(boxes.len() + 1, rng.gen_range(0..boxes.len() * 2)) as u16,
        );
        assert!(outputs > 0);
        let parameters = Parameters::default();
        let sufficient_amount =
            ErgoBox::MAX_BOX_SIZE as u64 * parameters.min_value_per_byte() as u64;
        assert!(sufficient_amount * (outputs as u64) < input_sum);
        let mut output_preamounts = vec![sufficient_amount; outputs as usize];
        let mut remainder = input_sum - sufficient_amount * outputs as u64;
        // TODO: find a smarter way to do this since sometimes number of iterations can blow up
        while remainder > 0 {
            let idx = rng.gen_range(0..output_preamounts.len());
            if remainder < input_sum / boxes.len() as u64 {
                output_preamounts[idx] = output_preamounts[idx].checked_add(remainder).unwrap();
                remainder = 0;
            } else {
                let val = rng.gen_range(0..=remainder);
                output_preamounts[idx] = output_preamounts[idx].checked_add(val).unwrap();
                remainder -= val;
            }
        }

        let mut token_amounts: Vec<HashMap<TokenId, u64>> = vec![HashMap::new(); outputs as usize];
        let mut available_token_slots = (outputs * 255) as usize;
        while !assets_map.is_empty() && available_token_slots > 0 {
            let cur = assets_map
                .iter()
                .map(|(&token_id, &token_amount)| (token_id, token_amount))
                .next()
                .unwrap();
            let out_idx = loop {
                let idx = rng.gen_range(0..token_amounts.len());
                if token_amounts[idx].len() < 255 {
                    break idx;
                }
            };
            let contains = token_amounts[out_idx].contains_key(&cur.0);

            let amount = if *cur.1.as_u64() == 1
                || (available_token_slots < assets_map.len() * 2 && !contains)
                || rng.gen()
            {
                *cur.1.as_u64()
            } else {
                rng.gen_range(1..=*cur.1.as_u64())
            };
            if amount == *cur.1.as_u64() {
                assets_map.remove(&cur.0);
            } else {
                assets_map.entry(cur.0).and_modify(|amt| {
                    *amt = amt
                        .checked_sub(&TokenAmount::try_from(amount).unwrap())
                        .unwrap()
                });
            }
            token_amounts[out_idx]
                .entry(cur.0)
                .and_modify(|token_amount| *token_amount += amount)
                .or_insert_with(|| {
                    available_token_slots -= 1;
                    amount
                });
        }
        let output_boxes = output_preamounts
            .into_iter()
            .zip(token_amounts)
            .map(|(amount, tokens)| -> (u64, Option<BoxTokens>) {
                (
                    amount,
                    tokens
                        .into_iter()
                        .map(|(token_id, token_amount)| {
                            Token::from((token_id, TokenAmount::try_from(token_amount).unwrap()))
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .ok(),
                )
            })
            .map(|(amount, tokens)| ErgoBoxCandidate {
                value: BoxValue::new(amount).unwrap(),
                ergo_tree: output_prop.clone(),
                tokens,
                additional_registers: NonMandatoryRegisters::empty(),
                creation_height: 0,
            })
            .collect();
        UnsignedTransaction::new_from_vec(
            boxes
                .iter()
                .map(|b| UnsignedInput::new(b.box_id(), ContextExtension::empty()))
                .collect(),
            vec![],
            output_boxes,
        )
        .unwrap()
    }
    fn valid_transaction_from_boxes(
        rng: TestRng,
        boxes: Vec<ErgoBox>,
        issue_new_token: bool,
        output_prop: ErgoTree,
        data_boxes: Vec<ErgoBox>,
    ) -> Transaction {
        let unsigned_tx = valid_unsigned_transaction_from_boxes(
            rng,
            &boxes,
            issue_new_token,
            output_prop,
            &data_boxes,
        );
        let tx_context =
            TransactionContext::new(unsigned_tx.clone(), boxes.clone(), data_boxes).unwrap();
        let wallet = Wallet::from_secrets(vec![]);
        let state_context = force_any_val();
        // Attempt to sign a transaction. If signing fails because script reduces to false or prover doesn't know some secret then return an invalid transaction
        wallet
            .sign_transaction(tx_context, &state_context, None)
            .or_else(|_| {
                Transaction::new(
                    TxIoVec::from_vec(
                        boxes
                            .iter()
                            .map(|b| Input {
                                box_id: b.box_id(),
                                spending_proof: ProverResult {
                                    proof: ProofBytes::Empty,
                                    extension: ContextExtension::empty(),
                                },
                            })
                            .collect(),
                    )
                    .unwrap(),
                    unsigned_tx.data_inputs,
                    unsigned_tx.output_candidates,
                )
            })
            .unwrap()
    }
    fn valid_transaction_gen_with_tree(
        tree: ErgoTree,
    ) -> impl Strategy<Value = (Vec<ErgoBox>, Transaction)> {
        let box_generator = gen_boxes(1, 100, 1, 100, Just(tree.clone()), None);
        (box_generator, bool::arbitrary()).prop_perturb(move |(boxes, issue_new_token), rng| {
            (
                boxes.clone(),
                valid_transaction_from_boxes(rng, boxes, issue_new_token, tree.clone(), vec![]),
            )
        })
    }

    fn valid_transaction_generator() -> impl Strategy<Value = (Vec<ErgoBox>, Transaction)> {
        let true_tree = ErgoTree::new(
            ErgoTreeHeader::v0(true),
            &Expr::Const(Constant {
                tpe: ergotree_ir::types::stype::SType::SBoolean,
                v: Literal::Boolean(true),
            }),
        )
        .unwrap();
        valid_transaction_gen_with_tree(true_tree)
    }

    fn update_asset<F: FnOnce(TokenAmount) -> TokenAmount>(
        transaction: &mut Transaction,
        boxes: &[ErgoBox],
        f: F,
    ) {
        for output in transaction.outputs.iter_mut() {
            if let Some(token) = output
                .tokens
                .iter_mut()
                .flatten()
                .find(|t| t.token_id != boxes[0].box_id().into())
            {
                token.amount = f(token.amount);
                break;
            }
        }
    }

    proptest! {
    #[test]
    // Test that a valid transaction is valid
    fn test_valid_transaction((boxes, tx) in valid_transaction_generator()) {
        let state_context = force_any_val();
        let tx_context = TransactionContext::new(tx, boxes, vec![]).unwrap();
        tx_context.validate(&state_context).unwrap();
    }
    #[test]
    fn test_ergo_preservation((mut boxes, mut tx) in valid_transaction_generator(), positive_delta: bool, change_output: bool) {
        let state_context = force_any_val();

        let box_value = if change_output {
            let slice: &mut [ErgoBox] = tx.outputs.as_mut();
            &mut slice[0].value
        }
        else {
            &mut boxes[0].value
        };
        if positive_delta {
            *box_value = box_value.checked_add(&BoxValue::SAFE_USER_MIN).unwrap();
        }
        else {
            *box_value = box_value.checked_sub(&BoxValue::SAFE_USER_MIN).unwrap();
        }

        assert!(tx.validate_stateless().is_ok());

        let tx_context = TransactionContext::new(tx, boxes, vec![]).unwrap();
        match tx_context.validate(&state_context) {
            Err(TxValidationError::ErgPreservationError(_, _)) => {},
            e => panic!("Expected validation to fail got {e:?}")
        }
    }
    #[test]
    fn test_zero_asset_creation((boxes, mut tx) in valid_transaction_generator()) {
        let state_context = force_any_val();
        update_asset(&mut tx, &boxes, |amount| amount.checked_add(&TokenAmount::MIN).unwrap());
        assert!(tx.validate_stateless().is_ok());

        let tx_context = TransactionContext::new(tx, boxes, vec![]).unwrap();
        match tx_context.validate(&state_context) {
            Err(TxValidationError::TokenPreservationError { .. } ) => {},
            other => panic!("Expected validation to fail, got {other:?}")
        }
    }
    #[test]
    fn test_asset_preservation((boxes, mut tx) in valid_transaction_generator()) {
        let state_context = force_any_val();
        update_asset(&mut tx, &boxes, |amount| amount.checked_add(&TokenAmount::MIN).unwrap());
        assert!(tx.validate_stateless().is_ok());

        let tx_context = TransactionContext::new(tx, boxes, vec![]).unwrap();
        match tx_context.validate(&state_context) {
            Err(TxValidationError::TokenPreservationError { .. } ) => {},
            other => panic!("Expected validation to fail, got {other:?}")
        }
    }
    }
    // Test that unspendable boxes can't be included in a transaction
    // TODO: When sigma-rust lands support for storage rent transactions, there should be a test that successfully passes validation when box is old enough
    #[test]
    fn test_false_proposition() {
        let state_context = force_any_val();
        let false_tree = ErgoTree::new(
            ErgoTreeHeader::v0(true),
            &Expr::Const(Constant {
                tpe: ergotree_ir::types::stype::SType::SBoolean,
                v: Literal::Boolean(false),
            }),
        )
        .unwrap();
        proptest!(|((boxes, tx) in valid_transaction_gen_with_tree(false_tree))| {
            assert!(tx.validate_stateless().is_ok());

            let tx_context = TransactionContext::new(tx, boxes, vec![]).unwrap();
            match tx_context.validate(&state_context) {
                Err(TxValidationError::ReducedToFalse(_, _)) => {},
                other => panic!("Expected validation to fail, got {other:?}")
            }
        });
    }
    #[test]
    fn test_monotonic_box_creation() {
        let true_tree = ErgoTree::new(
            ErgoTreeHeader::v0(true),
            &Expr::Const(Constant {
                tpe: ergotree_ir::types::stype::SType::SBoolean,
                v: Literal::Boolean(true),
            }),
        )
        .unwrap();

        let state_context_tx_gen = |tx: &Transaction, version| {
            let height = tx
                .output_candidates
                .iter()
                .map(|b| b.creation_height)
                .max()
                .unwrap();
            dbg!(height);
            let mut state_context: ErgoStateContext = force_any_val();
            state_context.pre_header.height = height;
            state_context.pre_header.version = version;
            state_context
        };
        let box_gen = gen_boxes(
            5,
            10,
            5,
            10,
            Just(true_tree.clone()),
            Some((0..i32::MAX as u32).boxed()),
        );
        // Generate a list of boxes. If monotonic_valid is true then monotonic height validation will pass, otherwise it will fail in tests
        let tx_gen =
            (box_gen, bool::arbitrary()).prop_perturb(|(boxes, monotonic_valid), mut rng| {
                let max_height = boxes.iter().map(|b| b.creation_height).max().unwrap();
                let mut unsigned_tx = valid_unsigned_transaction_from_boxes(
                    rng.clone(),
                    &boxes,
                    true,
                    true_tree.clone(),
                    &[],
                );
                if monotonic_valid {
                    unsigned_tx
                        .output_candidates
                        .iter_mut()
                        .for_each(|b| b.creation_height = max_height + rng.gen_range(1..1000));
                } else {
                    unsigned_tx.output_candidates.iter_mut().for_each(|b| {
                        b.creation_height = max_height.saturating_sub(rng.gen_range(1..1000))
                    });
                }
                let wallet = Wallet::from_secrets(vec![]);
                let state_context = force_any_val();
                let tx_context =
                    TransactionContext::new(unsigned_tx, boxes.clone(), vec![]).unwrap();
                let signed_tx = wallet
                    .sign_transaction(tx_context, &state_context, None)
                    .unwrap();
                (boxes, signed_tx, monotonic_valid)
            });
        proptest!(|((boxes, tx, monotonic_valid) in tx_gen)| {
            assert!(tx.validate_stateless().is_ok());

            // For blocks V1 and V2 monotonic height rule is not respected.
            let context1 = state_context_tx_gen(&tx, 1);
            let context2 = state_context_tx_gen(&tx, 2);
            // V3 enforces monotonic height rule, thus validation should fail if !monotonic_valid
            let context3 = state_context_tx_gen(&tx, 3);
            let tx_context = TransactionContext::new(tx, boxes, vec![]).unwrap();
            match tx_context.validate(&context1) {
                Ok(_) => {},
                other => panic!("Expected validation to succeed, got {other:?}")
            }
            match tx_context.validate(&context2) {
                Ok(_) => {},
                other => panic!("Expected validation to succeed, got {other:?}")
            }
            match (monotonic_valid, tx_context.validate(&context3)) {
                (true, Ok(())) => {},
                (false, Err(TxValidationError::MonotonicHeightError(_, _))) => {},
                other => panic!("Expected validation to fail, got {other:?}")
            }
        });
    }
}
