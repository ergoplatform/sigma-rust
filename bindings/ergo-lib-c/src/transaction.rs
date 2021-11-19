//! Ergo transaction

use ergo_lib_c_core::{transaction::*, Error, ErrorPtr};

use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    tx_id_ptr: *mut *const c_char,
) -> ErrorPtr {
    let res = match unsigned_tx_id(unsigned_tx_ptr) {
        Ok(s) => {
            *tx_id_ptr = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_from_json(
    json_str: *const c_char,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = unsigned_tx_from_json(&json, unsigned_tx_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_to_json(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match unsigned_tx_to_json(unsigned_tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_unsigned_tx_delete(ptr: UnsignedTransactionPtr) {
    unsafe { delete_ptr(ptr) }
}
