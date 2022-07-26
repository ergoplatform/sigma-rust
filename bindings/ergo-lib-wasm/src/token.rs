//! Token types

use std::convert::TryFrom;

use bounded_vec::BoundedVec;
use ergo_lib::ergo_chain_types::Base16DecodedBytes;
use ergo_lib::ergo_chain_types::Digest32;
use ergo_lib::ergotree_ir::chain;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::ergo_box::BoxId;
use crate::error_conversion::to_js;
use crate::json::TokenJsonEip12;
use crate::utils::I64;

/// A Bounded Vector for Tokens. A Box can have between 1 and ErgoBox::MAX_TOKENS_COUNT tokens
pub type BoxTokens = BoundedVec<Token, 1, { chain::ergo_box::ErgoBox::MAX_TOKENS_COUNT }>;
/// Token id (32 byte digest)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenId(chain::token::TokenId);

#[wasm_bindgen]
impl TokenId {
    /// Create token id from ergo box id (32 byte digest)
    pub fn from_box_id(box_id: &BoxId) -> TokenId {
        let box_id: chain::ergo_box::BoxId = box_id.clone().into();
        TokenId(chain::token::TokenId::from(box_id))
    }

    /// Parse token id (32 byte digest) from base16-encoded string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<TokenId, JsValue> {
        Base16DecodedBytes::try_from(str.to_string())
            .map_err(to_js)
            .and_then(|bytes| Digest32::try_from(bytes).map_err(to_js))
            .map(|dig| dig.into())
            .map(TokenId)
    }

    /// Base16 encoded string
    pub fn to_str(&self) -> String {
        self.0.clone().into()
    }

    /// Returns byte array (32 bytes)
    pub fn as_bytes(&self) -> Uint8Array {
        Uint8Array::from(self.0.as_ref())
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
    /// Create from i64 with bounds check
    pub fn from_i64(v: &I64) -> Result<TokenAmount, JsValue> {
        Ok(Self(
            chain::token::TokenAmount::try_from(i64::from(v.clone()) as u64).map_err(to_js)?,
        ))
    }

    /// Get value as signed 64-bit long (I64)
    pub fn as_i64(&self) -> I64 {
        i64::from(self.0).into()
    }

    /// big-endian byte array representation
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_u64().to_be_bytes().to_vec()
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

    /// Get token id
    pub fn id(&self) -> TokenId {
        TokenId(self.0.token_id.clone())
    }

    /// Get token amount
    pub fn amount(&self) -> TokenAmount {
        TokenAmount(self.0.amount)
    }

    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string_pretty(&self.0.clone())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    /// (similar to [`Self::to_json`], but as JS object with token amount encoding as string)
    pub fn to_js_eip12(&self) -> Result<JsValue, JsValue> {
        let t_dapp: TokenJsonEip12 = self.0.clone().into();
        JsValue::from_serde(&t_dapp).map_err(|e| JsValue::from_str(&format!("{}", e)))
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
pub struct Tokens(Option<BoxTokens>);

#[wasm_bindgen]
impl Tokens {
    /// Create empty Tokens
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Tokens(None)
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.as_ref().map(BoxTokens::len).unwrap_or(0)
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> Result<Token, JsValue> {
        Ok(self
            .0
            .as_ref()
            .ok_or_else::<JsValue, _>(|| "Tokens::get: no tokens available".into())?
            .get(index)
            .ok_or_else::<JsValue, _>(|| "".into())?
            .clone())
    }

    /// Adds an elements to the collection
    pub fn add(&mut self, elem: &Token) -> Result<(), JsValue> {
        if self.0.is_some() {
            #[allow(clippy::unwrap_used)]
            let mut new_vec = self.0.as_ref().unwrap().as_vec().clone();

            if new_vec.len() >= chain::ergo_box::ErgoBox::MAX_TOKENS_COUNT {
                return Err(
                    "Tokens::add: can't have more than ErgoBox::MAX_TOKENS_COUNT tokens".into(),
                );
            }

            new_vec.push(elem.clone());
            #[allow(clippy::unwrap_used)]
            let box_tokens = BoxTokens::from_vec(new_vec).unwrap();
            self.0 = Some(box_tokens);
        } else {
            self.0 = Some(BoxTokens::from([elem.clone()]));
        }
        Ok(())
    }
}

// impl From<Tokens> for Option<chain::ergo_box::BoxTokens> {
//     fn from(v: Tokens) -> Self {
//         chain::ergo_box::BoxTokens::from_vec(
//             v.0.iter().flatten().cloned().map(Into::into).collect(),
//         )
//         .ok()
//     }
// }
impl From<Option<chain::ergo_box::BoxTokens>> for Tokens {
    fn from(v: Option<chain::ergo_box::BoxTokens>) -> Self {
        Tokens(v.map(|t| t.mapped(Token)))
    }
}
