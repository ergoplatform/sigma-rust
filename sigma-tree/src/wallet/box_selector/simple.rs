//! Box selector which selects all provided inputs

use crate::chain::ergo_box::box_value::BoxValue;
use crate::chain::ergo_box::ErgoBoxAssets;
use crate::chain::token::TokenAmount;

use super::{BoxSelection, BoxSelector};

#[allow(dead_code)]
/// Selects all provided inputs
pub struct SimpleBoxSelector {}

impl<T: ErgoBoxAssets> BoxSelector<T> for SimpleBoxSelector {
    fn select(
        &self,
        inputs: Vec<T>,
        target_balance: BoxValue,
        target_tokens: &[TokenAmount],
    ) -> Result<BoxSelection<T>, super::BoxSelectorError> {
        // TODO: check if inputs have enough assets
        let len = inputs.len();
        Ok(BoxSelection {
            boxes: inputs.into_iter().take(len).collect(),
            // TODO: calculate change
            change_boxes: vec![],
        })
    }
}
