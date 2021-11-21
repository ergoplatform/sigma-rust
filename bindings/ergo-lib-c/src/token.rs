//! Token types
use ergo_lib_c_core::{ergo_box::ConstBoxIdPtr, token::*, Error};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{delete_ptr, ErrorPtr};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_token_id_from_box_id(
    box_id_ptr: ConstBoxIdPtr,
    token_id_out: *mut TokenIdPtr,
) -> ErrorPtr {
    let res = token_id_from_box_id(box_id_ptr, token_id_out);
    Error::c_api_from(res)
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
) -> ErrorPtr {
    let res = match token_id_to_str(token_id_ptr) {
        Ok(s) => {
            *_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_token_id_delete(ptr: TokenIdPtr) {
    unsafe { delete_ptr(ptr) }
}
