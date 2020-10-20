//! Box selection for transaction inputs

mod simple;
pub use simple::*;

use crate::chain::ergo_box::box_value::BoxValueError;
use crate::chain::ergo_box::ErgoBoxAssetsData;
use crate::chain::token::TokenId;
use crate::chain::{
    ergo_box::{box_value::BoxValue, ErgoBoxAssets},
    token::Token,
};
use thiserror::Error;

/// Selected boxes (by [`BoxSelector`])
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxSelection<T: ErgoBoxAssets> {
    /// Selected boxes to spend as transaction inputs
    pub boxes: Vec<T>,
    /// box assets with returning change amounts (to be put in tx outputs)
    pub change_boxes: Vec<ErgoBoxAssetsData>,
}

/// Box selector
pub trait BoxSelector<T: ErgoBoxAssets> {
    /// Selects boxes out of the provided inputs to satisfy target balance and tokens
    /// `inputs` - spendable boxes
    /// `target_balance` - value (in nanoERGs) to find in input boxes (inputs)
    /// `target_tokens` - token amounts to find in input boxes(inputs)
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

    /// Not enough tokens
    #[error("Not enough tokens (token id: {token_id:?}, {missing_amount} are missing)")]
    NotEnoughTokens {
        /// token id
        token_id: TokenId,
        /// missing(not enough) token amount
        missing_amount: u64,
    },

    /// BoxValue out of bounds
    #[error("BoxValue out of bounds")]
    BoxValueError(BoxValueError),
}

impl From<BoxValueError> for BoxSelectorError {
    fn from(e: BoxValueError) -> Self {
        BoxSelectorError::BoxValueError(e)
    }
}
