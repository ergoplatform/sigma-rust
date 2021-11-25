//! Token types

use std::convert::TryFrom;

use bounded_vec::BoundedVec;
use ergo_lib::ergotree_ir::chain::{self, base16_bytes::Base16DecodedBytes, digest32::Digest32};

use crate::{
    ergo_box::ConstBoxIdPtr,
    json::TokenJsonEip12,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Token id (32 byte digest)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenId(pub(crate) chain::token::TokenId);
pub type TokenIdPtr = *mut TokenId;
pub type ConstTokenIdPtr = *const TokenId;

/// Create token id from erbo box id (32 byte digest)
pub unsafe fn token_id_from_box_id(
    box_id_ptr: ConstBoxIdPtr,
    token_id_out: *mut TokenIdPtr,
) -> Result<(), Error> {
    let box_id = const_ptr_as_ref(box_id_ptr, "box_id_ptr")?;
    let token_id_out = mut_ptr_as_mut(token_id_out, "token_id_out")?;
    *token_id_out = Box::into_raw(Box::new(TokenId(chain::token::TokenId::from(
        box_id.0.clone(),
    ))));
    Ok(())
}

/// Parse token id (32 byte digest) from base16-encoded string
pub unsafe fn token_id_from_str(str: &str, token_id_out: *mut TokenIdPtr) -> Result<(), Error> {
    let token_id_out = mut_ptr_as_mut(token_id_out, "token_id_out")?;
    let base_16_decoded_bytes = Base16DecodedBytes::try_from(str.to_string())
        .map_err(|_| Error::Misc("TokenId: can't decode from base16 encoded string".into()))?;

    let token_id = Digest32::try_from(base_16_decoded_bytes)
        .map_err(|_| Error::Misc("TokenId: can't convert bytes into Digest32".into()))
        .map(|dig| dig.into())
        .map(TokenId)?;
    *token_id_out = Box::into_raw(Box::new(token_id));
    Ok(())
}

/// Base16 encoded string
pub unsafe fn token_id_to_str(token_id_ptr: ConstTokenIdPtr) -> Result<String, Error> {
    let token_id = const_ptr_as_ref(token_id_ptr, "token_id_ptr")?;
    Ok(token_id.0.clone().into())
}

/// Token amount with bound checks
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenAmount(pub(crate) chain::token::TokenAmount);
pub type TokenAmountPtr = *mut TokenAmount;
pub type ConstTokenAmountPtr = *const TokenAmount;

pub unsafe fn token_amount_from_i64(
    amount: i64,
    token_amount_out: *mut TokenAmountPtr,
) -> Result<(), Error> {
    let token_amount_out = mut_ptr_as_mut(token_amount_out, "token_amount_out")?;
    let inner = chain::token::TokenAmount::try_from(amount as u64)
        .map_err(|_| Error::Misc("TokenAmount: can't parse from i64".into()))?;
    *token_amount_out = Box::into_raw(Box::new(TokenAmount(inner)));
    Ok(())
}

/// Get value as signed 64-bit long
pub unsafe fn token_amount_as_i64(token_amount_ptr: ConstTokenAmountPtr) -> Result<i64, Error> {
    let token_amount = const_ptr_as_ref(token_amount_ptr, "token_amount_ptr")?;
    Ok(i64::from(token_amount.0))
}

/// Token represented with token id paired with its amount
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token(pub(crate) chain::token::Token);
pub type TokenPtr = *mut Token;
pub type ConstTokenPtr = *const Token;

/// Create a token with given token id and amount
pub unsafe fn token_new(
    token_id_ptr: ConstTokenIdPtr,
    token_amount_ptr: ConstTokenAmountPtr,
    token_out: *mut TokenPtr,
) -> Result<(), Error> {
    let token_id = const_ptr_as_ref(token_id_ptr, "token_id_ptr")?;
    let token_amount = const_ptr_as_ref(token_amount_ptr, "token_amount_ptr")?;
    let token_out = mut_ptr_as_mut(token_out, "token_out")?;
    *token_out = Box::into_raw(Box::new(Token(chain::token::Token {
        token_id: token_id.0.clone(),
        amount: token_amount.0,
    })));
    Ok(())
}

/// Get token id
pub unsafe fn token_get_id(
    token_ptr: ConstTokenPtr,
    token_id_out: *mut TokenIdPtr,
) -> Result<(), Error> {
    let token = const_ptr_as_ref(token_ptr, "token_ptr")?;
    let token_id_out = mut_ptr_as_mut(token_id_out, "token_id_out")?;
    *token_id_out = Box::into_raw(Box::new(TokenId(token.0.token_id.clone())));
    Ok(())
}

/// Get token amount
pub unsafe fn token_get_amount(
    token_ptr: ConstTokenPtr,
    token_amount_out: *mut TokenAmountPtr,
) -> Result<(), Error> {
    let token = const_ptr_as_ref(token_ptr, "token_ptr")?;
    let token_amount_out = mut_ptr_as_mut(token_amount_out, "token_amount_out")?;
    *token_amount_out = Box::into_raw(Box::new(TokenAmount(token.0.amount)));
    Ok(())
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
pub unsafe fn token_to_json_eip12(token_ptr: ConstTokenPtr) -> Result<String, Error> {
    let token = const_ptr_as_ref(token_ptr, "token_ptr")?;
    let t_dapp: TokenJsonEip12 = token.0.clone().into();
    serde_json::to_string(&t_dapp)
        .map_err(|_| Error::Misc("Token: can't serialize into JSON EIP-12".into()))
}

/// A Bounded Vector for Tokens. A Box can have between 1 and 255 tokens
pub type BoxTokens = BoundedVec<Token, 1, 255>;

/// Array of tokens. Note that we're not using `crate::collections::Collection` here due to the
/// use of the `BoundedVec`.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tokens(pub(crate) Option<BoxTokens>);
pub type TokensPtr = *mut Tokens;
pub type ConstTokensPtr = *const Tokens;

pub unsafe fn tokens_new(tokens_out: *mut TokensPtr) -> Result<(), Error> {
    let tokens_out = mut_ptr_as_mut(tokens_out, "tokens_out")?;
    *tokens_out = Box::into_raw(Box::new(Tokens(None)));
    Ok(())
}

pub unsafe fn tokens_len(tokens_ptr: ConstTokensPtr) -> Result<usize, Error> {
    let tokens = const_ptr_as_ref(tokens_ptr, "tokens_ptr")?;
    Ok(tokens.0.as_ref().map(BoxTokens::len).unwrap_or(0))
}

/// If token at given index exists, allocate a copy and store in `token_out` and return `Ok(true)`.
/// If token doesn't exist at the given index return Ok(false).
pub unsafe fn tokens_get(
    tokens_ptr: ConstTokensPtr,
    index: usize,
    token_out: *mut TokenPtr,
) -> Result<bool, Error> {
    let tokens = const_ptr_as_ref(tokens_ptr, "tokens_ptr")?;
    let token_out = mut_ptr_as_mut(token_out, "token_out")?;
    if let Some(tokens) = tokens.0.as_ref() {
        if let Some(token) = tokens.get(index) {
            *token_out = Box::into_raw(Box::new(token.clone()));
            return Ok(true);
        }
    }
    Ok(false)
}

pub unsafe fn tokens_add(tokens_ptr: TokensPtr, token_ptr: ConstTokenPtr) -> Result<(), Error> {
    let tokens = mut_ptr_as_mut(tokens_ptr, "tokens_ptr")?;
    let token = const_ptr_as_ref(token_ptr, "token_ptr")?;
    if tokens.0.is_some() {
        let mut new_vec = tokens.0.as_ref().unwrap().as_vec().clone();
        if new_vec.len() == 255 {
            return Err(Error::Misc(
                "Tokens.add: cannot have more than 255 tokens".into(),
            ));
        } else {
            new_vec.push(token.clone());
            tokens.0 = Some(BoxTokens::from_vec(new_vec).unwrap());
        }
    } else {
        tokens.0 = Some(BoxTokens::from([token.clone()]));
    }
    Ok(())
}
