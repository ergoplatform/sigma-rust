//! Box selector which selects all provided inputs

use std::marker::PhantomData;

use crate::chain::ergo_box::ErgoBoxAssets;

use super::{BoxSelection, BoxSelector};

/// Selects all provided inputs
pub struct SelectAllBoxSelector<T: ErgoBoxAssets> {
    a: PhantomData<T>,
}

impl<T: ErgoBoxAssets> SelectAllBoxSelector<T> {
    /// Create new selector
    pub fn new() -> SelectAllBoxSelector<T> {
        SelectAllBoxSelector {
            a: PhantomData::default(),
        }
    }
}

impl<T: ErgoBoxAssets> BoxSelector for SelectAllBoxSelector<T> {
    type Item = T;

    fn select(
        &self,
        inputs: Vec<T>,
        target_balance: crate::chain::ergo_box::box_value::BoxValue,
        target_tokens: &[crate::chain::token::TokenAmount],
    ) -> Result<super::BoxSelection<Self::Item>, super::BoxSelectorError> {
        // TODO: check if inputs have enough assets
        Ok(BoxSelection {
            boxes: inputs,
            // TODO: calculate change
            change_boxes: vec![],
        })
    }
}
