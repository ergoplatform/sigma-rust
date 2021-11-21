//! Token types

use std::convert::TryFrom;

use ergo_lib::ergotree_ir::chain::{self, base16_bytes::Base16DecodedBytes, digest32::Digest32};

use crate::{
    ergo_box::ConstBoxIdPtr,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Token id (32 byte digest)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenId(chain::token::TokenId);
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
pub struct TokenAmount(chain::token::TokenAmount);
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
pub struct Token(chain::token::Token);
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
