use crate::token_id::TokenId;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TokenInfo {
    pub token_id: TokenId,
    pub amount: u64,
}
