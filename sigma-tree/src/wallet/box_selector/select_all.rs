//! Box selector which selects all provided inputs

use std::marker::PhantomData;

use crate::chain::ergo_box::ErgoBoxAssets;

use super::{BoxSelection, BoxSelector};

#[allow(dead_code)]
/// Selects all provided inputs
pub struct SelectAllBoxSelector<T: ErgoBoxAssets> {
    a: PhantomData<T>,
}

impl<T: ErgoBoxAssets> SelectAllBoxSelector<T> {
    /// Create new selector
    pub fn new() -> Self {
        let _: Vec<T> = vec![];
        SelectAllBoxSelector { a: PhantomData }
    }
}

impl<T: ErgoBoxAssets> BoxSelector<T> for SelectAllBoxSelector<T> {
    fn select(
        &self,
        inputs: Vec<T>,
        target_balance: crate::chain::ergo_box::box_value::BoxValue,
        target_tokens: &[crate::chain::token::TokenAmount],
    ) -> Result<super::BoxSelection<T>, super::BoxSelectorError> {
        // TODO: check if inputs have enough assets
        Ok(BoxSelection {
            boxes: inputs,
            // TODO: calculate change
            change_boxes: vec![],
        })
    }
}
