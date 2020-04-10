use crate::ergo_tree::ErgoTree;
use std::collections::HashMap;

pub struct BoxId(Box<[u8]>);
pub struct TokenId(Box<[u8]>);

pub struct TokenInfo {
    pub token_id: TokenId,
    pub amount: u64,
}

pub struct NonMandatoryRegisterId(u8);

pub struct ErgoBoxCandidate {
    pub value: u64,
    pub ergo_tree: ErgoTree,
    pub tokens: Vec<TokenInfo>,
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<[u8]>>,
    pub creation_height: u32,
}
