//! Token types

use std::convert::TryFrom;

use ergo_lib::chain;
use ergo_lib::chain::Base16DecodedBytes;
use ergo_lib::chain::Digest32;
use wasm_bindgen::prelude::*;

use crate::ergo_box::BoxId;
use crate::utils::I64;

/// Token id (32 byte digest)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenId(chain::token::TokenId);

#[wasm_bindgen]
impl TokenId {
    /// Create token id from erbo box id (32 byte digest)
    pub fn from_box_id(box_id: &BoxId) -> TokenId {
        let box_id: chain::ergo_box::box_id::BoxId = box_id.clone().into();
        TokenId(chain::token::TokenId::from(box_id))
    }

    /// Parse token id (32 byte digets) from base16-encoded string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<TokenId, JsValue> {
        Base16DecodedBytes::try_from(str.to_string())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .and_then(|bytes| {
                Digest32::try_from(bytes).map_err(|e| JsValue::from_str(&format!("{}", e)))
            })
            .map(chain::token::TokenId)
            .map(TokenId)
    }
}

impl From<TokenId> for chain::token::TokenId {
    fn from(t_id: TokenId) -> Self {
        t_id.0
    }
}

/// Token amount with bound checks
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenAmount(chain::token::TokenAmount);

#[wasm_bindgen]
impl TokenAmount {
    /// Create from u32 with bounds check
    pub fn from_u32(v: u32) -> Result<TokenAmount, JsValue> {
        Ok(Self(
            chain::token::TokenAmount::try_from(v as u64)
                .map_err(|e| JsValue::from_str(&format!("{}", e)))?,
        ))
    }

    /// Get value as signed 64-bit long (I64)
    pub fn as_i64(&self) -> I64 {
        i64::from(self.0).into()
    }
}

impl From<TokenAmount> for chain::token::TokenAmount {
    fn from(ta: TokenAmount) -> Self {
        ta.0
    }
}

/// Token represented with token id paired with it's amount
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token(chain::token::Token);

#[wasm_bindgen]
impl Token {
    /// Create a token with given token id and amount
    #[wasm_bindgen(constructor)]
    pub fn new(token_id: &TokenId, amount: &TokenAmount) -> Self {
        Token(chain::token::Token {
            token_id: token_id.clone().into(),
            amount: amount.clone().into(),
        })
    }
}

impl From<Token> for chain::token::Token {
    fn from(t: Token) -> Self {
        t.0
    }
}

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
