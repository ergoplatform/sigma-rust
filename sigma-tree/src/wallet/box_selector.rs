//! Box selection for transaction inputs

// TODO: remove after the implementation
#![allow(unused_variables)]

use crate::chain::{
    ergo_box::{box_value::BoxValue, ErgoBoxAssets},
    token::TokenAmount,
};

/// Selected boxes (by [`BoxSelector`])
pub struct BoxSelection<T: ErgoBoxAssets> {
    /// selected boxes to spend
    pub boxes: Vec<T>,
    /// box assets with returning change amounts (to be put in tx outputs)
    pub change_boxes: Vec<T>,
}

/// Box selector
pub trait BoxSelector {
    /// Selects boxes out of the provided inputs to satisfy target balance and tokens
    fn select<T: ErgoBoxAssets>(
        inputs: &[T],
        target_balance: BoxValue,
        target_tokens: &[TokenAmount],
    ) -> Result<BoxSelection<T>, BoxSelectorError> {
        todo!()
    }
}

/// Errors of BoxSelector
pub enum BoxSelectorError {}
