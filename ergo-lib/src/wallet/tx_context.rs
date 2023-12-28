//! Transaction context

use ergotree_ir::chain::ergo_box::ErgoBox;
use thiserror::Error;

use crate::chain::transaction::ergo_transaction::ErgoTransaction;
use crate::chain::transaction::TransactionError;
use crate::ergotree_ir::chain::ergo_box::BoxId;
use ergotree_interpreter::eval::context::TxIoVec;

/// Transaction and an additional info required for signing
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TransactionContext<T: ErgoTransaction> {
    /// Unsigned transaction to sign
    pub spending_tx: T,
    /// Boxes corresponding to [`crate::chain::transaction::unsigned::UnsignedTransaction::inputs`]
    boxes_to_spend: TxIoVec<ErgoBox>,
    /// Boxes corresponding to [`crate::chain::transaction::unsigned::UnsignedTransaction::data_inputs`]
    pub(crate) data_boxes: Option<TxIoVec<ErgoBox>>,
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
        for (i, unsigned_input) in spending_tx.inputs_ids().enumerated() {
            if !boxes_to_spend.iter().any(|b| unsigned_input == b.box_id()) {
                return Err(TransactionContextError::InputBoxNotFound(i));
            }
        }
        let data_boxes_len = data_boxes.len();
        let data_boxes = if !data_boxes.is_empty() {
            Some(
                TxIoVec::from_vec(data_boxes)
                    .map_err(|_| TransactionContextError::TooManyDataInputBoxes(data_boxes_len))?,
            )
        } else {
            None
        };

        if let Some(data_inputs) = spending_tx.data_inputs().as_ref() {
            if let Some(data_boxes) = data_boxes.as_ref() {
                for (i, data_input) in data_inputs.iter().enumerate() {
                    if !data_boxes.iter().any(|b| data_input.box_id == b.box_id()) {
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
        })
    }

    /// Returns box with given id, if it exists.
    pub fn get_input_box(&self, box_id: &BoxId) -> Option<ErgoBox> {
        self.boxes_to_spend
            .iter()
            .find(|b| b.box_id() == *box_id)
            .cloned()
    }
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
