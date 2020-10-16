//! Token types

use ergo_lib::chain;
use wasm_bindgen::prelude::*;

/// Token id (32 byte digest)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenId(chain::token::TokenId);

/// Token amount with bound checks
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenAmount(chain::token::TokenAmount);

/// Token represented with token id paired with it's amount
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token(chain::token::Token);

/// Array of tokens
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tokens(Vec<Token>);

#[wasm_bindgen]
impl Tokens {
    /// Create empty Tokens
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Tokens(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> Token {
        self.0[index].clone()
    }

    /// Adds an elements to the collection
    pub fn add(&mut self, elem: &Token) {
        self.0.push(elem.clone());
    }
}

impl From<Tokens> for Vec<chain::token::Token> {
    fn from(v: Tokens) -> Self {
        v.0.iter().map(|i| i.0.clone()).collect()
    }
}
