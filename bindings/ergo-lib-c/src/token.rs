//! Token types
use ergo_lib_c_core::{ergo_box::ConstBoxIdPtr, token::*, Error};
use paste::paste;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{delete_ptr, ErrorPtr, ReturnOption};

// `TokenId` bindings ------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_id_from_box_id(
    box_id_ptr: ConstBoxIdPtr,
    token_id_out: *mut TokenIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    token_id_from_box_id(box_id_ptr, token_id_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_id_from_str(
    bytes_ptr: *const c_char,
    token_id_out: *mut TokenIdPtr,
) -> ErrorPtr {
    let str = CStr::from_ptr(bytes_ptr).to_string_lossy();
    let res = token_id_from_str(&str, token_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_id_to_str(
    token_id_ptr: ConstTokenIdPtr,
    _str: *mut *const c_char,
) {
    #[allow(clippy::unwrap_used)]
    {
        let s = token_id_to_str(token_id_ptr).unwrap();
        *_str = CString::new(s).unwrap().into_raw();
    }
}

#[no_mangle]
pub extern "C" fn ergo_wallet_token_id_delete(ptr: TokenIdPtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(TokenId);

// `TokenAmount` bindings --------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_amount_from_i64(
    amount: i64,
    token_amount_out: *mut TokenAmountPtr,
) -> ErrorPtr {
    let res = token_amount_from_i64(amount, token_amount_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_amount_as_i64(
    token_amount_ptr: ConstTokenAmountPtr,
) -> i64 {
    #[allow(clippy::unwrap_used)]
    token_amount_as_i64(token_amount_ptr).unwrap()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_token_amount_delete(ptr: TokenAmountPtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(TokenAmount);

// `Token` bindings --------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_new(
    token_id_ptr: ConstTokenIdPtr,
    token_amount_ptr: ConstTokenAmountPtr,
    token_out: *mut TokenPtr,
) {
    #[allow(clippy::unwrap_used)]
    token_new(token_id_ptr, token_amount_ptr, token_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_get_id(
    token_ptr: ConstTokenPtr,
    token_id_out: *mut TokenIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    token_get_id(token_ptr, token_id_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_get_amount(
    token_ptr: ConstTokenPtr,
    token_amount_out: *mut TokenAmountPtr,
) {
    #[allow(clippy::unwrap_used)]
    token_get_amount(token_ptr, token_amount_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_to_json_eip12(
    token_ptr: ConstTokenPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match token_to_json_eip12(token_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_token_delete(ptr: TokenPtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(Token);

// `Tokens` bindings -------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tokens_new(tokens_out: *mut TokensPtr) {
    #[allow(clippy::unwrap_used)]
    tokens_new(tokens_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tokens_len(tokens_ptr: ConstTokensPtr) -> usize {
    #[allow(clippy::unwrap_used)]
    tokens_len(tokens_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tokens_get(
    tokens_ptr: ConstTokensPtr,
    index: usize,
    token_out: *mut TokenPtr,
) -> ReturnOption {
    match tokens_get(tokens_ptr, index, token_out) {
        Ok(is_some) => ReturnOption {
            is_some,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnOption {
            is_some: false, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tokens_add(
    token_ptr: ConstTokenPtr,
    tokens_ptr: TokensPtr,
) -> ErrorPtr {
    Error::c_api_from(tokens_add(tokens_ptr, token_ptr))
}

#[no_mangle]
pub extern "C" fn ergo_wallet_tokens_delete(ptr: TokensPtr) {
    unsafe { delete_ptr(ptr) }
}
