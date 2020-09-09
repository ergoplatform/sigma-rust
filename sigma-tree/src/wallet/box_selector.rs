//! Box selection for transaction inputs

// TODO: remove after the implementation
#![allow(unused_variables)]

pub mod select_all;

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
    /// Type on inputs and the resulting selection
    type Item: ErgoBoxAssets;

    /// Selects boxes out of the provided inputs to satisfy target balance and tokens
    fn select(
        &self,
        inputs: Vec<Self::Item>,
        target_balance: BoxValue,
        target_tokens: &[TokenAmount],
    ) -> Result<BoxSelection<Self::Item>, BoxSelectorError> {
        todo!()
    }
}

/// Errors of BoxSelector
pub enum BoxSelectorError {}
