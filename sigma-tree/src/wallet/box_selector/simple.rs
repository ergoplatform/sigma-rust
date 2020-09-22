//! Box selector which selects all provided inputs

use std::convert::TryInto;

use crate::chain::ergo_box::box_value::BoxValue;
use crate::chain::ergo_box::ErgoBoxAssets;
use crate::chain::ergo_box::ErgoBoxAssetsData;
use crate::chain::token::TokenAmount;

use super::BoxSelectorError;
use super::{BoxSelection, BoxSelector};

#[allow(dead_code)]
/// Selects all provided inputs
pub struct SimpleBoxSelector {}

impl SimpleBoxSelector {
    /// Create new boxed instance
    pub fn new() -> Box<Self> {
        Box::new(SimpleBoxSelector {})
    }
}

impl<T: ErgoBoxAssets> BoxSelector<T> for SimpleBoxSelector {
    /// Selects inputs to satisfy target balance and tokens.
    /// `inputs` - available inputs (returns an error, if empty),
    /// `target_balance` - coins (in nanoERGs) needed,
    /// `target_tokens` - amount of tokens needed.
    /// Returns selected inputs and box assets(value+tokens) with change.
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
        let change_boxes: Vec<ErgoBoxAssetsData> = if unmet_target_balance == 0 {
            vec![]
        } else {
            let change_value: BoxValue = unmet_target_balance.abs().try_into()?;
            vec![ErgoBoxAssetsData {
                value: change_value,
                tokens: vec![],
            }]
        };
        Ok(BoxSelection {
            boxes: selected_inputs,
            change_boxes,
        })
    }
}

impl Default for SimpleBoxSelector {
    fn default() -> Self {
        SimpleBoxSelector {}
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::ergo_box::box_value;
    use crate::chain::ergo_box::ErgoBox;
    use proptest::{collection::vec, prelude::*};

    use super::*;

    #[test]
    fn test_empty_inputs() {
        let s = SimpleBoxSelector::new();
        let inputs: Vec<ErgoBox> = vec![];
        let r = s.select(inputs, BoxValue::MIN, vec![].as_slice());
        assert!(r.is_err());
    }

    proptest! {
        #[test]
        fn test_select(inputs in vec(any_with::<ErgoBox>((9000..10000000).into()), 1..10)) {
            let s = SimpleBoxSelector::new();
            let all_inputs_val = box_value::sum(inputs.iter().map(|b| b.value)).unwrap();

            let balance_too_much = all_inputs_val.checked_add(&BoxValue::MIN).unwrap();
            prop_assert!(s.select(inputs.clone(), balance_too_much, vec![].as_slice()).is_err());

            let balance_exact = all_inputs_val;
            let selection_exact = s.select(inputs.clone(), balance_exact, vec![].as_slice()).unwrap();
            prop_assert!(selection_exact.change_boxes.is_empty());
            prop_assert!(selection_exact.boxes == inputs);

            let balance_less = all_inputs_val.checked_sub(&BoxValue::MIN).unwrap();
            let selection_less = s.select(inputs.clone(), balance_less, vec![].as_slice()).unwrap();
            let expected_change_box = ErgoBoxAssetsData {value: BoxValue::MIN, tokens: vec![]};
            prop_assert!(selection_less.change_boxes == vec![expected_change_box]);
            prop_assert!(selection_less.boxes == inputs);
        }
    }
}
