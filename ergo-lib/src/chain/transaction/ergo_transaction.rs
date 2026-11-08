//! Exposes common properties for signed and unsigned transactions
use ergotree_interpreter::sigma_protocol::verifier::{VerificationResult, VerifierError};
pub use ergotree_ir::chain::context::ContextExtensionProvider;
use ergotree_ir::chain::context_extension::ContextExtension;
use ergotree_ir::{
    chain::{
        ergo_box::{box_value::BoxValue, BoxId, ErgoBox},
        token::{TokenAmountError, TokenId},
    },
    serialization::SigmaSerializationError,
};
use thiserror::Error;

use crate::wallet::tx_context::TransactionContextError;

use super::{unsigned::UnsignedTransaction, DataInput, Transaction};

/// Errors when validating transaction
#[derive(Error, Debug)]
pub enum TxValidationError {
    /// Transaction has more than [`i16::MAX`] inputs
    #[error("Sum of ERG in outputs overflowed")]
    /// Sum of ERG in outputs has overflowed
    OutputSumOverflow,
    /// Sum of ERG in inputs has overflowed
    #[error("Sum of ERG in inputs has overflowed")]
    InputSumOverflow,
    /// Token Amount Error
    #[error("Token amount is not valid, {0}")]
    TokenAmountError(#[from] TokenAmountError),
    #[error("Unique inputs: {0}, actual inputs: {1}")]
    /// The transaction is attempting to spend the same [`BoxId`] twice
    DoubleSpend(usize, usize),
    #[error("ERG value not preserved, input amount: {0}, output amount: {1}")]
    /// The amount of Ergo in inputs must be equal to the amount of ergo in output (cannot be burned)
    ErgPreservationError(u64, u64),
    #[error("Token preservation error for {token_id:?}, in amount: {in_amount:?}, out_amount: {out_amount:?}, allowed new token id: {new_token_id:?}")]
    /// Transaction is creating more tokens than exists in inputs. This is only allowed when minting a new token
    TokenPreservationError {
        /// If the transaction is minting a new token, then it must have this token id
        new_token_id: TokenId,
        /// The token id whose amount was not preserved
        token_id: TokenId,
        /// Total amount of token in inputs
        in_amount: u64,
        /// Total amount of token in outputs
        out_amount: u64,
    },
    #[error("Output {0} is dust, amount {1:?} < minimum {2}")]
    /// Transaction was creating a dust output. The value of a box should be >= than box size * [Parameters::min_value_per_byte](crate::chain::parameters::Parameters::min_value_per_byte())
    DustOutput(BoxId, BoxValue, u64),
    #[error("Creation height {0} > preheader height")]
    /// The output's height is greater than the current block height
    InvalidHeightError(u32),
    #[error("Creation height {0} <= input box max height{1}")]
    /// After Block V3, all output boxes height must be >= max(inputs.height). See <https://github.com/ergoplatform/eips/blob/master/eip-0039.md> for more information
    MonotonicHeightError(u32, u32),
    #[error("Output box's creation height is negative (not allowed after block version 1)")]
    /// Negative heights are not allowed after block v1.
    /// When using sigma-rust where heights are always unsigned, this error may be because creation height was set to be >= 2147483648
    NegativeHeight,
    #[error("Output box size {0} > maximum {max_size}", max_size = ErgoBox::MAX_BOX_SIZE)]
    /// Box size is > [ErgoBox::MAX_BOX_SIZE]
    BoxSizeExceeded(usize),
    #[error("Output box size {0} > maximum {max_size}", max_size = ErgoBox::MAX_SCRIPT_SIZE)]
    /// Script size is > [ErgoBox::MAX_SCRIPT_SIZE]
    ScriptSizeExceeded(usize),
    #[error("TX context error: {0}")]
    /// Transaction Context Error
    TransactionContextError(#[from] TransactionContextError),
    /// Input's proposition reduced to false. This means the proof provided for the input was most likely invalid
    #[error("Input {0} reduced to false during verification: {1:?}")]
    ReducedToFalse(usize, VerificationResult),
    /// Serialization error
    #[error("Sigma serialization error: {0}")]
    SigmaSerializationError(#[from] SigmaSerializationError),
    /// Verifying input script failed
    #[error("Verifier error on input {0}: {1}")]
    VerifierError(usize, VerifierError),
    /// Transaction initialization cost (INTERPRETER_INIT_COST + per-input/output/
    /// data-input/token structural cost) already exceeds the per-tx JIT cost limit
    /// before any input script evaluation begins. The u64 is the init cost in
    /// JitCost units (block cost × 10).
    #[error("Init cost {0} exceeds tx cost limit")]
    InitCostExceeded(u64),
}

/// Exposes common properties for signed and unsigned transactions
pub trait ErgoTransaction: ContextExtensionProvider {
    /// input boxes ids
    fn inputs_ids(&self) -> impl ExactSizeIterator<Item = BoxId>;
    /// data input boxes
    fn data_inputs(&self) -> Option<&[DataInput]>;
    /// output boxes
    fn outputs(&self) -> &[ErgoBox];

    /// Stateless transaction validation (no blockchain context) for a transaction
    /// Returns [Ok(())] if validation has succeeded or returns [`TxValidationError`]
    fn validate_stateless(&self) -> Result<(), TxValidationError> {
        // Note that we don't need to check if inputs/data inputs/outputs are >= 1 <= 32767 here since BoundedVec takes care of that
        let inputs = self.inputs_ids();
        let outputs = self.outputs();

        outputs
            .iter()
            .try_fold(0i64, |a, b| a.checked_add(b.value.as_i64()))
            .ok_or(TxValidationError::OutputSumOverflow)?;

        // Check if there are no double-spends in input (one BoxId being spent more than once)
        let len = inputs.len();
        let unique_count = inputs.collect::<hashbrown::HashSet<_>>().len();
        if unique_count != len {
            return Err(TxValidationError::DoubleSpend(unique_count, len));
        }
        Ok(())
    }
}

impl ErgoTransaction for UnsignedTransaction {
    fn inputs_ids(&self) -> impl ExactSizeIterator<Item = BoxId> {
        self.inputs.iter().map(|input| input.box_id)
    }

    fn data_inputs(&self) -> Option<&[DataInput]> {
        self.data_inputs.as_ref().map(|di| di.as_slice())
    }

    fn outputs(&self) -> &[ErgoBox] {
        self.outputs.as_slice()
    }
}

impl ErgoTransaction for Transaction {
    fn inputs_ids(&self) -> impl ExactSizeIterator<Item = BoxId> {
        self.inputs.iter().map(|input| input.box_id)
    }

    fn data_inputs(&self) -> Option<&[DataInput]> {
        self.data_inputs.as_ref().map(|di| di.as_slice())
    }

    fn outputs(&self) -> &[ErgoBox] {
        self.outputs.as_slice()
    }
}

impl ContextExtensionProvider for UnsignedTransaction {
    fn context_extension(&self, input_index: usize) -> Option<&ContextExtension> {
        self.inputs.get(input_index).map(|input| &input.extension)
    }
}

impl ContextExtensionProvider for Transaction {
    fn context_extension(&self, input_index: usize) -> Option<&ContextExtension> {
        self.inputs
            .get(input_index)
            .map(|input| &input.spending_proof.extension)
    }
}
