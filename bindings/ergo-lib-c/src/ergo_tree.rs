//! ErgoTree

use ergo_lib_c_core::{
    constant::{ConstConstantPtr, Constant, ConstantPtr},
    ergo_tree::*,
    Error,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{delete_ptr, ErrorPtr, ReturnNum, ReturnOption};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_from_base16_bytes(
    bytes_ptr: *const c_char,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let bytes_str = CStr::from_ptr(bytes_ptr).to_string_lossy();
    let res = ergo_tree_from_base16_bytes(&bytes_str, ergo_tree_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let res = ergo_tree_from_bytes(bytes_ptr, len, ergo_tree_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_bytes_len(
    ergo_tree_ptr: ConstErgoTreePtr,
) -> ReturnNum<usize> {
    match ergo_tree_bytes_len(ergo_tree_ptr) {
        Ok(value) => ReturnNum {
            value,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnNum {
            value: 0, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_to_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = ergo_tree_to_bytes(ergo_tree_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_to_base16_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    _str: *mut *const c_char,
) -> ErrorPtr {
    let res = match ergo_tree_to_base16_bytes(ergo_tree_ptr) {
        Ok(s) => {
            *_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_constants_len(
    ergo_tree_ptr: ConstErgoTreePtr,
) -> ReturnNum<usize> {
    match ergo_tree_constants_len(ergo_tree_ptr) {
        Ok(value) => ReturnNum {
            value,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnNum {
            value: 0, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_get_constant(
    ergo_tree_ptr: ConstErgoTreePtr,
    index: usize,
    constant_out: *mut ConstantPtr,
) -> ReturnOption<Constant> {
    match ergo_tree_get_constant(ergo_tree_ptr, index, constant_out) {
        Ok(is_some) => ReturnOption {
            is_some,
            value_ptr: constant_out,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnOption {
            is_some: false, // Just a dummy value
            value_ptr: constant_out,
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_with_constant(
    ergo_tree_ptr: ConstErgoTreePtr,
    index: usize,
    constant_ptr: ConstConstantPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let res = ergo_tree_with_constant(ergo_tree_ptr, index, constant_ptr, ergo_tree_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_template_bytes_len(
    ergo_tree_ptr: ConstErgoTreePtr,
) -> ReturnNum<usize> {
    match ergo_tree_template_bytes_len(ergo_tree_ptr) {
        Ok(value) => ReturnNum {
            value,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnNum {
            value: 0, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_tree_template_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = ergo_tree_template_bytes(ergo_tree_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_tree_delete(ptr: ErgoTreePtr) {
    unsafe { delete_ptr(ptr) }
}
