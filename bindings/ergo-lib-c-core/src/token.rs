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
    let token_id_ptr = const_ptr_as_ref(token_id_ptr, "token_id_ptr")?;
    Ok(token_id_ptr.0.clone().into())
}
