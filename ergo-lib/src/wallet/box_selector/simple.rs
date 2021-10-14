//! Naive box selector, collects inputs until target balance is reached

use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryInto;

use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::BoxTokens;
use ergotree_ir::chain::token::Token;
use ergotree_ir::chain::token::TokenAmount;
use ergotree_ir::chain::token::TokenId;

use crate::wallet::box_selector::sum_tokens;
use crate::wallet::box_selector::sum_tokens_from_boxes;
use crate::wallet::box_selector::ErgoBoxAssetsData;

use super::BoxSelectorError;
use super::ErgoBoxAssets;
use super::{BoxSelection, BoxSelector};

/// Simple box selector, collects inputs(sorted by targeted assets) until target balance is reached
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
        // sum all target tokens into hash map (think repeating token ids)
        let mut target_tokens_left: HashMap<TokenId, TokenAmount> = sum_tokens(Some(target_tokens));
        let mut has_value_change = false;
        let mut has_token_change = false;
        let mut sorted_inputs = inputs;
        sorted_inputs.sort_by(|a, b| {
            let a_target_tokens_count = a
                .tokens()
                .into_iter()
                .flatten()
                .filter(|t| target_tokens_left.contains_key(&t.token_id))
                .count();
            let b_target_tokens_count = b
                .tokens()
                .into_iter()
                .flatten()
                .filter(|t| target_tokens_left.contains_key(&t.token_id))
                .count();
            a_target_tokens_count.cmp(&b_target_tokens_count)
        });
        // reverse, so they'll be sorted by descending order (boxes with target tokens will be first)
        sorted_inputs.reverse();
        sorted_inputs.into_iter().for_each(|b| {
            let value_change_amt: u64 = if target_balance > selected_boxes_value {
                0
            } else {
                selected_boxes_value - target_balance
            };
            if target_balance > selected_boxes_value
                || (has_value_change || has_token_change)
                    && (value_change_amt < *BoxValue::SAFE_USER_MIN.as_u64())
                || (!target_tokens_left.is_empty()
                    && b.tokens()
                        .into_iter()
                        .flatten()
                        .any(|t| target_tokens_left.contains_key(&t.token_id)))
            {
                selected_boxes_value += u64::from(b.value());
                if selected_boxes_value > target_balance {
                    has_value_change = true;
                }
                let mut selected_tokens_from_this_box: HashMap<TokenId, TokenAmount> =
                    HashMap::new();
                b.tokens().into_iter().flatten().for_each(|t| {
                    if let Some(token_amount_left_to_select) =
                        target_tokens_left.get(&t.token_id).cloned()
                    {
                        let token_amount_in_box = t.amount;
                        if token_amount_left_to_select <= token_amount_in_box {
                            target_tokens_left.remove(&t.token_id);
                        } else {
                            target_tokens_left
                                .entry(t.token_id.clone())
                                .and_modify(|amt| {
                                    *amt = amt.checked_sub(&token_amount_in_box).unwrap()
                                });
                        }
                        let selected_token_amt =
                            min(token_amount_in_box, token_amount_left_to_select);
                        selected_tokens_from_this_box
                            .entry(t.token_id.clone())
                            .and_modify(|amt| {
                                *amt = amt.checked_add(&selected_token_amt).unwrap();
                            })
                            .or_insert(selected_token_amt);
                    }
                });
                if sum_tokens(b.tokens().as_ref().map(BoxTokens::as_ref))
                    != selected_tokens_from_this_box
                {
                    has_token_change = true;
                };
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
                target_tokens_left.into_iter().map(Token::from).collect(),
            ));
        }
        let change_boxes: Vec<ErgoBoxAssetsData> = if !has_value_change && !has_token_change {
            vec![]
        } else {
            let change_value: BoxValue = (selected_boxes_value - target_balance).try_into()?;
            let mut change_tokens = sum_tokens_from_boxes(selected_inputs.as_slice());
            target_tokens.iter().try_for_each(|t| {
                match change_tokens.get(&t.token_id).cloned() {
                    Some(selected_boxes_t_amt) if selected_boxes_t_amt == t.amount => {
                        change_tokens.remove(&t.token_id);
                        Ok(())
                    }
                    Some(selected_boxes_t_amt) if selected_boxes_t_amt > t.amount => {
                        change_tokens.insert(
                            t.token_id.clone(),
                            selected_boxes_t_amt.checked_sub(&t.amount).unwrap(),
                        );
                        Ok(())
                    }
                    _ => Err(BoxSelectorError::NotEnoughTokens(vec![t.clone()])),
                }
            })?;
            vec![ErgoBoxAssetsData {
                value: change_value,
                tokens: BoxTokens::from_vec(change_tokens.into_iter().map(Token::from).collect())
                    .ok(),
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
    use std::convert::TryFrom;

    use ergotree_ir::chain::ergo_box::box_value::checked_sum;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use proptest::{collection::vec, prelude::*};

    use crate::wallet::box_selector::sum_value;

    use super::*;

    #[test]
    fn test_empty_inputs() {
        let s = SimpleBoxSelector::new();
        let inputs: Vec<ErgoBox> = vec![];
        let r = s.select(inputs, BoxValue::SAFE_USER_MIN, vec![].as_slice());
        assert!(r.is_err());
    }

    proptest! {

        #[test]
        fn test_select_not_enough_value(inputs in
                                        vec(any_with::<ErgoBoxAssetsData>(
                                            (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10)) {
            let s = SimpleBoxSelector::new();
            let all_inputs_val = checked_sum(inputs.iter().map(|b| b.value)).unwrap();

            let balance_too_much = all_inputs_val.checked_add(&BoxValue::SAFE_USER_MIN).unwrap();
            prop_assert!(s.select(inputs, balance_too_much, vec![].as_slice()).is_err());
        }

        #[test]
        fn test_select_value(inputs in
                              vec(any_with::<ErgoBoxAssetsData>(
                              (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 2..10)) {
            let all_inputs_val = checked_sum(inputs.iter().map(|b| b.value)).unwrap();
            let s = SimpleBoxSelector::new();
            let target_balance = all_inputs_val.checked_sub(&(all_inputs_val.as_u64()/2).try_into().unwrap()).unwrap();
            let target_tokens = vec![];
            let selection = s.select(inputs, target_balance, target_tokens.as_slice()).unwrap();
            let out_box = ErgoBoxAssetsData {value: target_balance, tokens: target_tokens.try_into().ok()};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                            sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens")
        }

        #[test]
        fn test_select_change_value_is_too_small(inputs in
                                                 vec(any_with::<ErgoBoxAssetsData>(
                                                 (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 2..10)) {
            let first_input_box = inputs.get(0).unwrap().clone();
            let s = SimpleBoxSelector::new();
            let target_balance = BoxValue::try_from(first_input_box.value().as_u64() - 1).unwrap();
            let selection = s.select(inputs, target_balance, vec![].as_slice()).unwrap();
            prop_assert!(!selection.change_boxes.is_empty());
            let out_box = ErgoBoxAssetsData {value: target_balance, tokens: None};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                            sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens")
        }

        #[test]
        fn test_select_value_change_and_tokens(inputs in
                      vec(any_with::<ErgoBoxAssetsData>(
                      (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 2..10),
                        target_balance in
                        any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 1500).into())) {
            let first_input_box = inputs.get(0).unwrap().clone();
            prop_assume!(!first_input_box.tokens.is_none());
            let first_input_box_token = first_input_box.tokens.as_ref().unwrap().first();
            let first_input_box_token_amount = u64::from(first_input_box_token.amount);
            prop_assume!(first_input_box_token_amount > 1);
            let s = SimpleBoxSelector::new();
            let target_token_amount = first_input_box_token_amount / 2;
            let target_token_id = first_input_box_token.token_id.clone();
            let target_token = Token {token_id: target_token_id,
                                      amount: target_token_amount.try_into().unwrap()};
            let selection = s.select(inputs, target_balance, vec![target_token.clone()].as_slice()).unwrap();
            prop_assert!(!selection.change_boxes.is_empty());
            let out_box = ErgoBoxAssetsData {value: target_balance, tokens: vec![target_token].try_into().ok()};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                            sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens");
        }

        #[test]
        fn test_select_all_value(inputs in
                                  vec(any_with::<ErgoBoxAssetsData>(
                                      (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10)) {
            let s = SimpleBoxSelector::new();
            let all_inputs_val = checked_sum(inputs.iter().map(|b| b.value)).unwrap();
            let balance_less = all_inputs_val.checked_sub(&BoxValue::SAFE_USER_MIN).unwrap();
            let selection = s.select(inputs, balance_less, vec![].as_slice()).unwrap();
            let out_box = ErgoBoxAssetsData {value: balance_less, tokens: None};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                sum_value(change_boxes_plus_out.as_slice()),
                "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                "all tokens from selected boxes should equal all tokens from the change boxes + target tokens")
        }

        #[test]
        fn test_select_single_token(inputs in
                                    vec(any_with::<ErgoBoxAssetsData>(
                                        (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                    target_balance in
                                    any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 800).into()),
                                    target_token_amount in 1..100u64) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens_from_boxes(inputs.as_slice());
            prop_assume!(!all_input_tokens.is_empty());
            let target_token_id = all_input_tokens.keys().collect::<Vec<&TokenId>>().get((all_input_tokens.len() - 1) / 2)
                                                                                    .cloned().unwrap();
            let input_token_amount = *all_input_tokens.get(target_token_id).unwrap();
            let target_token_amount: TokenAmount = target_token_amount.try_into().unwrap();
            prop_assume!(input_token_amount >= target_token_amount);
            let target_token = Token {token_id: target_token_id.clone(), amount: target_token_amount};
            let selection = s.select(inputs, target_balance, vec![target_token.clone()].as_slice()).unwrap();
            let out_box = ErgoBoxAssetsData {value: target_balance, tokens: vec![target_token].try_into().ok()};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                            sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens");
            prop_assert!(
                selection.boxes.iter()
                    .all(|b| b.tokens().into_iter().flatten().any(|t| t.token_id == *target_token_id)),
                "only boxes that have target token should be selected, got: {0:?}", selection.boxes
            );
        }

        #[test]
        fn test_select_single_token_all_amount(inputs in
                                               vec(any_with::<ErgoBoxAssetsData>(
                                                   (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                               target_balance in
                                               any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 500).into())) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens_from_boxes(inputs.as_slice());
            prop_assume!(!all_input_tokens.is_empty());
            let target_token_id = all_input_tokens.keys().collect::<Vec<&TokenId>>().get((all_input_tokens.len() - 1) / 2)
                                                                                    .cloned().unwrap();
            let input_token_amount = *all_input_tokens.get(target_token_id).unwrap();
            let target_token = Token {token_id: target_token_id.clone(), amount: input_token_amount};
            let selection = s.select(inputs, target_balance, vec![target_token.clone()].as_slice()).unwrap();
            let out_box = ErgoBoxAssetsData {value: target_balance, tokens: vec![target_token].try_into().ok()};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                            sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens");
            prop_assert!(
                selection.boxes.iter()
                    .all(|b| b.tokens().into_iter().flatten().any(|t| t.token_id == *target_token_id)),
                "only boxes that have target token should be selected, got: {0:?}", selection.boxes
            );
        }

        #[test]
        fn test_select_multiple_tokens(inputs in
                                       vec(any_with::<ErgoBoxAssetsData>(
                                           (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                       target_balance in
                                       any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 500).into()),
                                       target_token1_amount in 1..100u64,
                                       target_token2_amount in 2..100u64) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens_from_boxes(inputs.as_slice());
            prop_assume!(all_input_tokens.len() >= 2);
            let all_input_tokens_keys = all_input_tokens.keys().collect::<Vec<&TokenId>>();
            let target_token1_id = all_input_tokens_keys.first().cloned().unwrap();
            let target_token2_id = all_input_tokens_keys.last().cloned().unwrap();
            let input_token1_amount = *all_input_tokens.get(target_token1_id).unwrap();
            let input_token2_amount = *all_input_tokens.get(target_token2_id).unwrap();
            prop_assume!(u64::from(input_token1_amount) >= target_token1_amount);
            prop_assume!(u64::from(input_token2_amount) >= target_token2_amount);
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
                                             tokens: BoxTokens::from_vec(vec![target_token1, target_token2_part1, target_token2_part2]).ok()};
            let mut change_boxes_plus_out = vec![out_box];
            change_boxes_plus_out.append(&mut selection.change_boxes.clone());
            prop_assert_eq!(sum_value(selection.boxes.as_slice()),
                            sum_value(change_boxes_plus_out.as_slice()),
                            "total value of the selected boxes should equal target balance + total value in change boxes");
            prop_assert_eq!(sum_tokens_from_boxes(selection.boxes.as_slice()),
                            sum_tokens_from_boxes(change_boxes_plus_out.as_slice()),
                            "all tokens from selected boxes should equal all tokens from the change boxes + target tokens");
            prop_assert!(
                selection.boxes.iter()
                    .all(|b| b.tokens().into_iter().flatten().any(|t| t.token_id == *target_token1_id || t.token_id == *target_token2_id)),
                "only boxes that have target tokens should be selected, got: {0:?}", selection.boxes
            );
        }

        #[test]
        fn test_select_not_enough_tokens(inputs in
                                         vec(any_with::<ErgoBoxAssetsData>(
                                             (BoxValue::MIN_RAW * 1000 .. BoxValue::MIN_RAW * 10000).into()), 1..10),
                                         target_balance in
                                         any_with::<BoxValue>((BoxValue::MIN_RAW * 100 .. BoxValue::MIN_RAW * 1000).into())) {
            let s = SimpleBoxSelector::new();
            let all_input_tokens = sum_tokens_from_boxes(inputs.as_slice());
            prop_assume!(!all_input_tokens.is_empty());
            let target_token_id = all_input_tokens.keys().collect::<Vec<&TokenId>>().get(0).cloned().unwrap();
            let input_token_amount = u64::from(*all_input_tokens.get(target_token_id).unwrap()) / 2;
            let target_token_amount = TokenAmount::MAX_RAW;
            prop_assume!(input_token_amount < target_token_amount);
            let target_token = Token {token_id: target_token_id.clone(), amount: target_token_amount.try_into().unwrap()};
            let selection = s.select(inputs, target_balance, vec![target_token].as_slice());
            prop_assert!(selection.is_err());
        }

    }
}
