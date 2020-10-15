//! Box selection for transaction inputs

pub mod simple;

use crate::chain::ergo_box::box_value::BoxValueError;
use crate::chain::ergo_box::ErgoBoxAssetsData;
use crate::chain::{
    ergo_box::{box_value::BoxValue, ErgoBoxAssets},
    token::Token,
};
use thiserror::Error;

/// Selected boxes (by [`BoxSelector`])
pub struct BoxSelection<T: ErgoBoxAssets> {
    /// selected boxes to spend
    pub boxes: Vec<T>,
    /// box assets with returning change amounts (to be put in tx outputs)
    pub change_boxes: Vec<ErgoBoxAssetsData>,
}

/// Box selector
pub trait BoxSelector<T: ErgoBoxAssets> {
    /// Selects boxes out of the provided inputs to satisfy target balance and tokens
    fn select(
        &self,
        inputs: Vec<T>,
        target_balance: BoxValue,
        target_tokens: &[Token],
    ) -> Result<BoxSelection<T>, BoxSelectorError>;
}

/// Errors of BoxSelector
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum BoxSelectorError {
    /// Not enough coins
    #[error("Not enough coins({0} nanoERGs are missing)")]
    NotEnoughCoins(u64),

    /// BoxValue out of bounds
    #[error("BoxValue out of bounds")]
    BoxValueError(BoxValueError),
}

impl From<BoxValueError> for BoxSelectorError {
    fn from(e: BoxValueError) -> Self {
        BoxSelectorError::BoxValueError(e)
    }
}
