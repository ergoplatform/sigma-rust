//! Box selection for transaction inputs

pub mod simple;

use crate::chain::{
    ergo_box::{box_value::BoxValue, ErgoBoxAssets},
    token::TokenAmount,
};
use thiserror::Error;

/// Selected boxes (by [`BoxSelector`])
pub struct BoxSelection<T: ErgoBoxAssets> {
    /// selected boxes to spend
    pub boxes: Vec<T>,
    /// box assets with returning change amounts (to be put in tx outputs)
    pub change_boxes: Vec<Box<dyn ErgoBoxAssets>>,
}

/// Box selector
pub trait BoxSelector<T: ErgoBoxAssets> {
    /// Selects boxes out of the provided inputs to satisfy target balance and tokens
    fn select(
        &self,
        inputs: Vec<T>,
        target_balance: BoxValue,
        target_tokens: &[TokenAmount],
    ) -> Result<BoxSelection<T>, BoxSelectorError>;
}

/// Errors of BoxSelector
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum BoxSelectorError {}
