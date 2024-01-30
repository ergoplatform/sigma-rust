//! Exposes common properties for signed and unsigned transactions
use ergotree_interpreter::{eval::context::TxIoVec, sigma_protocol::prover::ContextExtension};
use ergotree_ir::{
    chain::{
        ergo_box::{box_value::BoxValue, BoxId, ErgoBox},
        token::{TokenAmount, TokenAmountError, TokenId},
    },
    serialization::SigmaSerializationError,
};
use itertools::Itertools;
use thiserror::Error;

use crate::wallet::tx_context::TransactionContextError;

use super::{unsigned::UnsignedTransaction, DataInput, Transaction, TxVerifyError};

/// Errors when validating transaction
#[derive(Error, Debug)]
pub enum TxValidationError {
    /// Transaction has more than [`i16::MAX`] inputs
    /// Sum of ERG in outputs has overflowed
    #[error("Sum of ERG in outputs overflowed")]
    OutputSumOverflow,
    #[error("Sum of ERG in inputs has overflowed")]
    InputSumOverflow,
    #[error("Token amount is not valid, {0}")]
    TokenAmountError(#[from] TokenAmountError),
    /// The transaction is attempting to spend the same [`BoxId`] twice
    #[error("Unique inputs: {0}, actual inputs: {1}")]
    DoubleSpend(usize, usize),
    #[error("ERG value not preserved, input amount: {0}, output amount: {1}")]
    ErgPreservationError(u64, u64),
    #[error("Token preservation error for {token_id:?}, in amount: {in_amount:?}, out_amount: {out_amount:?}, allowed new token id: {new_token_id:?}")]
    TokenPreservationError {
        new_token_id: TokenId,
        token_id: TokenId,
        in_amount: u64,
        out_amount: u64,
    },
    #[error("Output {0} is dust, amount {1:?} < minimum {2}")]
    DustOutput(BoxId, BoxValue, u64),
    #[error("Creation height {0} <= {1}")]
    MonotonicHeightError(u32, u32),
    #[error("Output box's creation height is negative (not allowed after block version 1)")]
    NegativeHeight,
    #[error("Output box size {0} > maximum {}", ErgoBox::MAX_BOX_SIZE)]
    BoxSizeExceeded(usize),
    #[error("Output box size {0} > maximum {}", ErgoBox::MAX_SCRIPT_SIZE)]
    ScriptSizeExceeded(usize),
    #[error("TX context error: {0}")]
    TransactionContextError(TransactionContextError),
    // TODO: should probably merge TxValidationError and TxVerifyError
    #[error("Transaction verification error: {0}")]
    TransactionVerificationError(#[from] TxVerifyError),
    #[error("Input {0} reduced to false during verification")]
    ReducedToFalse(usize),
    #[error("Sigma serialization error: {0}")]
    SigmaSerializationError(#[from] SigmaSerializationError),
}

/// Exposes common properties for signed and unsigned transactions
pub trait ErgoTransaction {
    /// input boxes ids
    fn inputs_ids(&self) -> TxIoVec<BoxId>;
    /// data input boxes
    fn data_inputs(&self) -> Option<TxIoVec<DataInput>>;
    /// output boxes
    fn outputs(&self) -> TxIoVec<ErgoBox>;
    /// ContextExtension for the given input index
    fn context_extension(&self, input_index: usize) -> Option<ContextExtension>;

    /// Stateless transaction validation (no blockchain context) for a transaction
    /// Returns [`Ok(())`] if validation has succeeded or returns [`TxValidationError`]
    fn validate_stateless(&self) -> Result<(), TxValidationError> {
        // Note that we don't need to check if inputs/data inputs/outputs are >= 1 <= 32767 here since BoundedVec takes care of that
        let inputs = self.inputs_ids();
        let outputs = self.outputs();

        // TODO: simplify this once try_reduce is stable
        // TODO: Check if outputs are not dust (this should be done outside of validate_stateless since this depends on blockchain parameters)
        outputs
            .iter()
            .try_fold(0i64, |a, b| a.checked_add(b.value.as_i64()))
            .ok_or(TxValidationError::OutputSumOverflow)?;

        // Check if there are no double-spends in input (one BoxId being spent more than once)
        let unique_count = inputs.iter().unique().count();
        if unique_count != inputs.len() {
            return Err(TxValidationError::DoubleSpend(unique_count, inputs.len()));
        }
        Ok(())
    }
}

impl ErgoTransaction for UnsignedTransaction {
    fn inputs_ids(&self) -> TxIoVec<BoxId> {
        self.inputs.clone().mapped(|input| input.box_id)
    }

    fn data_inputs(&self) -> Option<TxIoVec<DataInput>> {
        self.data_inputs.clone()
    }

    fn outputs(&self) -> TxIoVec<ErgoBox> {
        #[allow(clippy::unwrap_used)] // box serialization cannot fail?
        self.output_candidates
            .clone()
            .enumerated()
            .try_mapped(|(idx, b)| ErgoBox::from_box_candidate(&b, self.id(), idx as u16))
            .unwrap()
    }

    fn context_extension(&self, input_index: usize) -> Option<ContextExtension> {
        self.inputs
            .get(input_index)
            .map(|input| input.extension.clone())
    }
}

impl ErgoTransaction for Transaction {
    fn inputs_ids(&self) -> TxIoVec<BoxId> {
        self.inputs.clone().mapped(|input| input.box_id)
    }

    fn data_inputs(&self) -> Option<TxIoVec<DataInput>> {
        self.data_inputs.clone()
    }

    fn outputs(&self) -> TxIoVec<ErgoBox> {
        self.outputs.clone()
    }

    fn context_extension(&self, input_index: usize) -> Option<ContextExtension> {
        self.inputs
            .get(input_index)
            .map(|input| input.spending_proof.extension.clone())
    }
}
