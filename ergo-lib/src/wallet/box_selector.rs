//! Box selection for transaction inputs

mod simple;

use std::collections::HashMap;

use bounded_vec::BoundedVec;
use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::BoxId;
use ergotree_ir::chain::ergo_box::BoxTokens;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::token::Token;
use ergotree_ir::chain::token::TokenAmount;
use ergotree_ir::chain::token::TokenAmountError;
use ergotree_ir::chain::token::TokenId;
pub use simple::*;

use thiserror::Error;

/// Bounded vec with minimum 1 element and max i16::MAX elements
pub type SelectedBoxes<T> = BoundedVec<T, 1, { i16::MAX as usize }>;

/// Selected boxes (by [`BoxSelector`])
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxSelection<T: ErgoBoxAssets> {
    /// Selected boxes to spend as transaction inputs
    pub boxes: SelectedBoxes<T>,
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

    /// Token amount err
    #[error("TokenAmountError: {0:?}")]
    TokenAmountError(#[from] TokenAmountError),

    /// CheckPreservationError
    #[error("CheckPreservationError: {0:?}")]
    CheckPreservation(#[from] CheckPreservationError),

    /// Not enough coins for change box
    #[error("Not enough coins for change box: {0:?}")]
    NotEnoughCoinsForChangeBox(#[from] NotEnoughCoinsForChangeBox),

    /// Selected inputs out of bounds
    #[error("Selected inputs out of bounds: {0}")]
    SelectedInputsOutOfBounds(usize),
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
pub fn sum_tokens(ts: Option<&[Token]>) -> Result<HashMap<TokenId, TokenAmount>, TokenAmountError> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    ts.into_iter().flatten().try_for_each(|t| {
        if let Some(amt) = res.get_mut(&t.token_id) {
            *amt = amt.checked_add(&t.amount)?;
        } else {
            res.insert(t.token_id, t.amount);
        }
        Ok(())
    })?;
    Ok(res)
}

/// Returns the total token amounts (all tokens combined) of the given boxes
pub fn sum_tokens_from_boxes<T: ErgoBoxAssets>(
    bs: &[T],
) -> Result<HashMap<TokenId, TokenAmount>, TokenAmountError> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    bs.iter().try_for_each(|b| {
        b.tokens().into_iter().flatten().try_for_each(|t| {
            if let Some(amt) = res.get_mut(&t.token_id) {
                *amt = amt.checked_add(&t.amount)?;
            } else {
                res.insert(t.token_id, t.amount);
            }

            Ok(())
        })
    })?;
    Ok(res)
}

/// Sums two hashmaps of tokens (summing amounts of the same token)
pub fn sum_tokens_from_hashmaps(
    tokens1: HashMap<TokenId, TokenAmount>,
    tokens2: HashMap<TokenId, TokenAmount>,
) -> Result<HashMap<TokenId, TokenAmount>, TokenAmountError> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    tokens1
        .into_iter()
        .chain(tokens2)
        .try_for_each(|(id, t_amt)| {
            if let Some(amt) = res.get_mut(&id) {
                *amt = amt.checked_add(&t_amt)?;
            } else {
                res.insert(id, t_amt);
            }
            Ok(())
        })?;
    Ok(res)
}

/// Subtract tokens2 from tokens1
/// subtracting amounts of the same token or removing the token if amount is the same
/// Returns an error if trying to subtract more tokens than there are in tokens1
pub fn subtract_tokens(
    tokens1: &HashMap<TokenId, TokenAmount>,
    tokens2: &HashMap<TokenId, TokenAmount>,
) -> Result<HashMap<TokenId, TokenAmount>, TokenAmountError> {
    let mut res: HashMap<TokenId, TokenAmount> = tokens1.clone();
    tokens2.iter().try_for_each(|(id, t_amt)| {
        if let Some(amt) = res.get_mut(id) {
            if amt == t_amt {
                res.remove(id);
            } else {
                *amt = amt.checked_sub(t_amt)?;
            }
        } else {
            // trying to subtract a token not found in tokens1
            return Err(TokenAmountError::OutOfBounds(-(*t_amt.as_u64() as i64)));
        }
        Ok(())
    })?;
    Ok(res)
}

/// Arbitrary impl for ErgoBoxAssetsData
#[allow(clippy::unwrap_used, clippy::panic)]
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use std::ops::Range;

    use ergotree_ir::chain::{
        ergo_box::{
            box_value::{arbitrary::ArbBoxValueRange, BoxValue},
            BoxTokens,
        },
        token::{arbitrary::ArbTokenIdParam, Token},
    };
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    use super::ErgoBoxAssetsData;

    /// Parameters for generating a token
    pub struct ArbTokensParam {
        /// Predefined or random token ids
        pub token_id_param: ArbTokenIdParam,
        /// how many distincts tokens to generate
        pub token_count_range: Range<usize>,
    }

    impl Default for ArbTokensParam {
        fn default() -> Self {
            ArbTokensParam {
                token_id_param: ArbTokenIdParam::default(),
                token_count_range: 0..3,
            }
        }
    }

    /// Parameters to generate ErgoBoxAssetsData
    #[derive(Default)]
    pub struct ArbErgoBoxAssetsDataParam {
        /// how many nanoERGs to generate
        pub value_range: ArbBoxValueRange,
        /// what and how many tokens to generate
        pub tokens_param: ArbTokensParam,
    }

    impl From<Range<u64>> for ArbErgoBoxAssetsDataParam {
        fn from(r: Range<u64>) -> Self {
            ArbErgoBoxAssetsDataParam {
                value_range: r.into(),
                tokens_param: ArbTokensParam::default(),
            }
        }
    }

    impl Arbitrary for ErgoBoxAssetsData {
        type Parameters = ArbErgoBoxAssetsDataParam;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<BoxValue>(args.value_range),
                vec(
                    any_with::<Token>(args.tokens_param.token_id_param),
                    args.tokens_param.token_count_range,
                ),
            )
                .prop_map(|(value, tokens)| Self {
                    value,
                    tokens: BoxTokens::from_vec(tokens).ok(),
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;

        fn arbitrary() -> Self::Strategy {
            Self::arbitrary_with(Default::default())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {

    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::BoxTokens;
    use ergotree_ir::chain::token::Token;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    use crate::wallet::box_selector::sum_tokens;
    use crate::wallet::box_selector::sum_tokens_from_boxes;

    use super::ErgoBoxAssetsData;

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
                    .unwrap()
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
