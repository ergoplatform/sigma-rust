//! Box selection for transaction inputs

mod simple;
use std::collections::HashMap;

use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::box_value::BoxValueError;
use ergotree_ir::chain::ergo_box::BoxId;
use ergotree_ir::chain::ergo_box::BoxTokens;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::token::Token;
use ergotree_ir::chain::token::TokenAmount;
use ergotree_ir::chain::token::TokenId;
pub use simple::*;

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
    #[error("Not enough tokens, missing {0:?}")]
    NotEnoughTokens(Vec<Token>),

    /// BoxValue out of bounds
    #[error("BoxValue out of bounds")]
    BoxValueError(BoxValueError),
}

impl From<BoxValueError> for BoxSelectorError {
    fn from(e: BoxValueError) -> Self {
        BoxSelectorError::BoxValueError(e)
    }
}

/// Assets that ErgoBox holds
pub trait ErgoBoxAssets {
    /// Box value
    fn value(&self) -> BoxValue;
    /// Tokens (ids and amounts)
    fn tokens(&self) -> Option<BoxTokens>;
}

/// Simple struct to hold ErgoBoxAssets values
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxAssetsData {
    /// Box value
    pub value: BoxValue,
    /// Tokens
    pub tokens: Option<BoxTokens>,
}

impl ErgoBoxAssets for ErgoBoxAssetsData {
    fn value(&self) -> BoxValue {
        self.value
    }

    fn tokens(&self) -> Option<BoxTokens> {
        self.tokens.clone()
    }
}

impl ErgoBoxAssets for ErgoBoxCandidate {
    fn value(&self) -> BoxValue {
        self.value
    }

    fn tokens(&self) -> Option<BoxTokens> {
        self.tokens.clone()
    }
}

impl ErgoBoxAssets for ErgoBox {
    fn value(&self) -> BoxValue {
        self.value
    }

    fn tokens(&self) -> Option<BoxTokens> {
        self.tokens.clone()
    }
}

/// id of the ergo box
pub trait ErgoBoxId {
    /// Id of the ergo box
    fn box_id(&self) -> BoxId;
}

impl ErgoBoxId for ErgoBox {
    fn box_id(&self) -> BoxId {
        self.box_id()
    }
}

/// Returns the total value of the given boxes
pub fn sum_value<T: ErgoBoxAssets>(bs: &[T]) -> u64 {
    bs.iter().map(|b| *b.value().as_u64()).sum()
}

/// Returns the total token amounts (all tokens combined)
pub fn sum_tokens(ts: Option<&[Token]>) -> HashMap<TokenId, TokenAmount> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    ts.into_iter().flatten().for_each(|t| {
        res.entry(t.token_id.clone())
            .and_modify(|amt| *amt = amt.checked_add(&t.amount).unwrap())
            .or_insert(t.amount);
    });
    res
}

/// Returns the total token amounts (all tokens combined) of the given boxes
pub fn sum_tokens_from_boxes<T: ErgoBoxAssets>(bs: &[T]) -> HashMap<TokenId, TokenAmount> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    bs.iter().for_each(|b| {
        b.tokens().into_iter().flatten().for_each(|t| {
            res.entry(t.token_id.clone())
                .and_modify(|amt| *amt = amt.checked_add(&t.amount).unwrap())
                .or_insert(t.amount);
        });
    });
    res
}

#[cfg(test)]
mod tests {

    use ergotree_ir::chain::ergo_box::box_value::arbitrary::ArbBoxValueRange;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::BoxTokens;
    use ergotree_ir::chain::token::Token;
    use proptest::{arbitrary::Arbitrary, collection::vec, option::of, prelude::*};
    use sigma_test_util::force_any_val;

    use crate::wallet::box_selector::sum_tokens;
    use crate::wallet::box_selector::sum_tokens_from_boxes;

    use super::ErgoBoxAssetsData;

    impl Arbitrary for ErgoBoxAssetsData {
        type Parameters = ArbBoxValueRange;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (any_with::<BoxValue>(args), of(vec(any::<Token>(), 0..3)))
                .prop_map(|(value, tokens)| Self {
                    value,
                    tokens: tokens.map(BoxTokens::from_vec).map(Result::ok).flatten(),
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;

        fn arbitrary() -> Self::Strategy {
            Self::arbitrary_with(Default::default())
        }
    }

    #[test]
    fn test_sum_tokens_repeating_token_id() {
        let token = force_any_val::<Token>();
        let b = ErgoBoxAssetsData {
            value: BoxValue::SAFE_USER_MIN,
            tokens: BoxTokens::from_vec(vec![token.clone(), token.clone()]).ok(),
        };
        assert_eq!(
            u64::from(
                *sum_tokens_from_boxes(vec![b.clone(), b].as_slice())
                    .get(&token.token_id)
                    .unwrap()
            ),
            u64::from(token.amount) * 4
        );
    }

    proptest! {

        #[test]
        fn sum_tokens_eq(b in any::<ErgoBoxAssetsData>()) {
            prop_assert_eq!(sum_tokens(b.tokens.as_ref().map(BoxTokens::as_ref)), sum_tokens_from_boxes(vec![b].as_slice()))
        }
    }
}
