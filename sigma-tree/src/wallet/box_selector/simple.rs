//! Box selector which selects all provided inputs

use crate::chain::ergo_box::box_value::BoxValue;
use crate::chain::ergo_box::ErgoBoxAssets;
use crate::chain::token::TokenAmount;

use super::BoxSelectorError;
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
    ) -> Result<BoxSelection<T>, BoxSelectorError> {
        assert!(target_tokens.is_empty(), "tokens are not yet supported");
        let mut selected_inputs: Vec<T> = vec![];
        let mut unmet_target_balance: i64 = target_balance.into();
        inputs.into_iter().for_each(|b| {
            if unmet_target_balance > 0 {
                let b_value: i64 = b.value().into();
                unmet_target_balance -= b_value;
                selected_inputs.push(b);
            };
        });
        if unmet_target_balance > 0 {
            return Err(BoxSelectorError::NotEnoughCoins(
                unmet_target_balance.abs() as u64
            ));
        }
        let _change = unmet_target_balance.abs();
        Ok(BoxSelection {
            boxes: selected_inputs,
            // TODO: make change "box"
            change_boxes: vec![],
        })
        // TODO: add tests
    }
}
