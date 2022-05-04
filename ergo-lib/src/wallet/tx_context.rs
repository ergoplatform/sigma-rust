//! Transaction context

use ergotree_ir::chain::ergo_box::ErgoBox;

use crate::ergotree_ir::chain::ergo_box::BoxId;
use ergotree_interpreter::eval::context::TxIoVec;

use super::signing::{ErgoTransaction, TxSigningError};

/// Transaction and an additional info required for signing
#[derive(PartialEq, Debug, Clone)]
pub struct TransactionContext<T: ErgoTransaction> {
    /// Unsigned transaction to sign
    pub spending_tx: T,
    /// Boxes corresponding to [`UnsignedTransaction::inputs`]
    boxes_to_spend: TxIoVec<ErgoBox>,
    /// Boxes corresponding to [`UnsignedTransaction::data_inputs`]
    pub(crate) data_boxes: Option<TxIoVec<ErgoBox>>,
}

impl<T: ErgoTransaction> TransactionContext<T> {
    /// New TransactionContext
    pub fn new(
        spending_tx: T,
        boxes_to_spend: TxIoVec<ErgoBox>,
        data_boxes: Option<TxIoVec<ErgoBox>>,
    ) -> Result<Self, TxSigningError> {
        for (i, unsigned_input) in spending_tx.inputs_ids().enumerated() {
            if !boxes_to_spend.iter().any(|b| unsigned_input == b.box_id()) {
                return Err(TxSigningError::InputBoxNotFound(i));
            }
        }
        if let Some(data_inputs) = spending_tx.data_inputs().as_ref() {
            if let Some(data_boxes) = data_boxes.as_ref() {
                for (i, data_input) in data_inputs.iter().enumerate() {
                    if !data_boxes.iter().any(|b| data_input.box_id == b.box_id()) {
                        return Err(TxSigningError::DataInputBoxNotFound(i));
                    }
                }
            } else {
                return Err(TxSigningError::DataInputBoxNotFound(0));
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
