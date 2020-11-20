use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_state_context::ErgoStateContext;
use crate::wallet::signing::TransactionContext;
use thiserror::Error;

use super::tvalue::SBoxT;
use super::tvalue::SCollT;
use super::tvalue::SIntT;
use super::tvalue::TValue;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
    pub height: TValue<SIntT>,
    pub self_box: TValue<SBoxT>,
    pub outputs: TValue<SCollT<SBoxT>>,
}

impl Context {
    /// Dummy instance intended for tests where actual values are not used
    #[cfg(test)]
    pub fn dummy() -> Self {
        use crate::test_util::force_any_val;
        Context {
            height: 0i32.into(),
            self_box: force_any_val::<ErgoBox>().into(),
            outputs: vec![force_any_val::<ErgoBox>()].into(),
        }
    }

    /// Create new instance:
    /// `self_index` - index of the SELF box in the tx_ctx.boxes_to_spend
    pub fn new(
        state_ctx: &ErgoStateContext,
        tx_ctx: &TransactionContext,
        self_index: usize,
    ) -> Result<Self, ContextError> {
        let height = state_ctx.pre_header.height.into();
        let self_box = tx_ctx
            .boxes_to_spend
            .get(self_index)
            .cloned()
            .ok_or(ContextError::SelfIndexOutOfBounds)?
            .into();
        let output_boxes: Vec<ErgoBox> = tx_ctx
            .spending_tx
            .output_candidates
            .iter()
            .enumerate()
            .map(|(idx, b)| ErgoBox::from_box_candidate(b, tx_ctx.spending_tx.id(), idx as u16))
            .collect();
        let outputs = output_boxes.into();
        Ok(Context {
            height,
            self_box,
            outputs,
        })
    }
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ContextError {
    /// self_index is out of bounds for TransactionContext::boxes_to_spend
    #[error("self_index is out of bounds for TransactionContext::boxes_to_spend")]
    SelfIndexOutOfBounds,
}
