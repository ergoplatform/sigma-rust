//! Naive box selector, collects inputs until target balance is reached

use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

use crate::chain::ergo_box::box_value::BoxValue;
use crate::chain::ergo_box::sum_tokens;
use crate::chain::ergo_box::ErgoBoxAssets;
use crate::chain::ergo_box::ErgoBoxAssetsData;
use crate::chain::token::Token;
use crate::chain::token::TokenAmount;
use crate::chain::token::TokenId;

use super::BoxSelectorError;
use super::{BoxSelection, BoxSelector};

/// Naive box selector, collects inputs until target balance is reached
#[allow(dead_code)]
pub struct SimpleBoxSelector {}

impl SimpleBoxSelector {
    /// Create new boxed instance
    pub fn new() -> Self {
        SimpleBoxSelector {}
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
        target_tokens: &[Token],
    ) -> Result<BoxSelection<T>, BoxSelectorError> {
        let mut selected_inputs: Vec<T> = vec![];
        let mut selected_boxes_value: u64 = 0;
        let target_balance: u64 = target_balance.into();
        let mut target_tokens_left: HashMap<TokenId, u64> = target_tokens
            .iter()
            .map(|t| (t.token_id.clone(), u64::from(t.amount)))
            .collect();
        inputs.into_iter().for_each(|b| {
            if target_balance > selected_boxes_value || !target_tokens_left.is_empty() {
                selected_boxes_value += u64::from(b.value());
                b.tokens().iter().for_each(|t| {
                    let token_amount_left = *target_tokens_left.get(&t.token_id).unwrap_or(&0);
                    let token_amount_in_box = u64::from(t.amount);
                    if token_amount_left <= token_amount_in_box {
                        target_tokens_left.remove(&t.token_id);
                    } else {
                        target_tokens_left
                            .insert(t.token_id.clone(), token_amount_left - token_amount_in_box);
                    }
                });
                selected_inputs.push(b);
            };
        });
        if selected_boxes_value < target_balance {
            return Err(BoxSelectorError::NotEnoughCoins(
                target_balance - selected_boxes_value,
            ));
        }
        if !target_tokens.is_empty() && !target_tokens_left.is_empty() {
            return Err(BoxSelectorError::NotEnoughTokens(
                target_tokens_left
                    .iter()
                    .map(|(token_id, token_amount)| Token {
                        token_id: token_id.clone(),
                        amount: TokenAmount::try_from(*token_amount).unwrap(),
                    })
                    .collect(),
            ));
        }
        // TODO: return error when no ERG change but tokens
        // TODO: what if ERG change < min box value but there is token change?
        let change_boxes: Vec<ErgoBoxAssetsData> =
            if selected_boxes_value == target_balance && target_tokens.is_empty() {
                vec![]
            } else {
                let change_value: BoxValue = (selected_boxes_value - target_balance).try_into()?;
                let mut change_tokens = sum_tokens(selected_inputs.as_slice());
                target_tokens.iter().for_each(|t| {
                    let selected_boxes_t_amt = change_tokens.get(&t.token_id).unwrap();
                    let t_change_amt = *selected_boxes_t_amt - u64::from(t.amount);
                    change_tokens.insert(t.token_id.clone(), t_change_amt);
                });
                vec![ErgoBoxAssetsData {
                    value: change_value,
                    tokens: change_tokens
                        .iter()
                        .map(|t| Token {
                            token_id: t.0.clone(),
                            amount: TokenAmount::try_from(*t.1).unwrap(),
                        })
                        .collect(),
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
    use crate::chain::ergo_box::sum_tokens;
    use crate::chain::ergo_box::sum_value;
    use crate::chain::ergo_box::ErgoBox;
    use crate::chain::ergo_box::ErgoBoxAssetsData;
    use proptest::{collection::vec, prelude::*};

    use super::*;

    #[test]
    fn test_empty_inputs() {
        let s = SimpleBoxSelector::new();
        let inputs: Vec<ErgoBox> = vec![];
        let r = s.select(inputs, BoxValue::SAFE_USER_MIN, vec![].as_slice());
        assert!(r.is_err());
    }

    // TODO: add test for repeated TokenId in target_tokens

    proptest! {

        #[test]
        fn test_select_not_enough_value(inputs in
                                        vec(any_with::<ErgoBoxAssetsData>(
                                            (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10)) {
            let s = SimpleBoxSelector::new();
            let all_inputs_val = box_value::checked_sum(inputs.iter().map(|b| b.value)).unwrap();

            let balance_too_much = all_inputs_val.checked_add(&BoxValue::SAFE_USER_MIN).unwrap();
            prop_assert!(s.select(inputs, balance_too_much, vec![].as_slice()).is_err());
        }

        #[test]
        fn test_select_only_value(inputs in
                                  vec(any_with::<ErgoBoxAssetsData>(
                                      (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10)) {
            let s = SimpleBoxSelector::new();
            let all_inputs_val = box_value::checked_sum(inputs.iter().map(|b| b.value)).unwrap();
            let balance_less = all_inputs_val.checked_sub(&BoxValue::SAFE_USER_MIN).unwrap();
            let selection_less = s.select(inputs.clone(), balance_less, vec![].as_slice()).unwrap();
            prop_assert!(selection_less.boxes == inputs);
            prop_assert_eq!(sum_value(selection_less.boxes.as_slice()),
                            balance_less.as_u64() + sum_value(selection_less.change_boxes.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens(selection_less.boxes.as_slice()),
                            sum_tokens(selection_less.change_boxes.as_slice()),
                            "all tokens from change boxes should equal all tokens from the input boxes");
        }

        #[test]
        fn test_select_single_token(inputs in
                                    vec(any_with::<ErgoBoxAssetsData>(
                                        (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                    target_balance in
                                    any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 1000).into()),
                                    target_token_amount in 1..100u64) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens(inputs.as_slice());
            prop_assume!(!all_input_tokens.is_empty());
            let target_token_id = all_input_tokens.keys().collect::<Vec<&TokenId>>().get(0).cloned().unwrap();
            let input_token_amount = *all_input_tokens.get(target_token_id).unwrap();
            prop_assume!(input_token_amount >= target_token_amount);
            let target_token = Token {token_id: target_token_id.clone(), amount: target_token_amount.try_into().unwrap()};
            let selection = s.select(inputs, target_balance, vec![target_token.clone()].as_slice()).unwrap();
            let out_box = ErgoBoxAssetsData {value: target_balance, tokens: vec![target_token]};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens(selection.boxes.as_slice()),
                            sum_tokens(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens");
        }

        #[test]
        fn test_select_multiple_tokens(inputs in
                                       vec(any_with::<ErgoBoxAssetsData>(
                                           (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                       target_balance in
                                       any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 1000).into()),
                                       target_token1_amount in 1..100u64,
                                       target_token2_amount in 2..100u64) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens(inputs.as_slice());
            prop_assume!(all_input_tokens.len() >= 2);
            let all_input_tokens_keys = all_input_tokens.keys().collect::<Vec<&TokenId>>();
            let target_token1_id = all_input_tokens_keys.get(0).cloned().unwrap();
            let target_token2_id = all_input_tokens_keys.get(1).cloned().unwrap();
            let input_token1_amount = *all_input_tokens.get(target_token1_id).unwrap();
            let input_token2_amount = *all_input_tokens.get(target_token2_id).unwrap();
            prop_assume!(input_token1_amount >= target_token1_amount);
            prop_assume!(input_token2_amount >= target_token2_amount);
            let target_token1 = Token {token_id: target_token1_id.clone(), amount: target_token1_amount.try_into().unwrap()};
            // simulate repeating token ids (e.g the same token id mentioned twice)
            let target_token2_amount_part1 = target_token2_amount / 2;
            let target_token2_amount_part2 = target_token2_amount - target_token2_amount_part1;
            let target_token2_part1 = Token {token_id: target_token2_id.clone(),
                                             amount: target_token2_amount_part1.try_into().unwrap()};
            let target_token2_part2 = Token {token_id: target_token2_id.clone(),
                                             amount: target_token2_amount_part2.try_into().unwrap()};
            let target_tokens = vec![target_token1.clone(), target_token2_part1.clone(), target_token2_part2.clone()];
            let selection = s.select(inputs, target_balance, target_tokens.as_slice()).unwrap();
            let out_box = ErgoBoxAssetsData {value: target_balance,
                                             tokens: vec![target_token1, target_token2_part1, target_token2_part2]};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens(selection.boxes.as_slice()),
                            sum_tokens(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens");
        }

        #[test]
        fn test_select_not_enough_tokens(inputs in
                                         vec(any_with::<ErgoBoxAssetsData>(
                                             (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                         target_balance in
                                         any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 1000).into())) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens(inputs.as_slice());
            prop_assume!(!all_input_tokens.is_empty());
            let target_token_id = all_input_tokens.keys().collect::<Vec<&TokenId>>().get(0).cloned().unwrap();
            let input_token_amount = all_input_tokens.get(target_token_id).unwrap() / 2;
            let target_token_amount = TokenAmount::MAX;
            prop_assume!(input_token_amount < target_token_amount);
            let target_token = Token {token_id: target_token_id.clone(), amount: target_token_amount.try_into().unwrap()};
            let selection = s.select(inputs, target_balance, vec![target_token].as_slice());
            prop_assert!(selection.is_err());
        }

    }
}
