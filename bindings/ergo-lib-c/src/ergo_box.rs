use ergo_lib_c_core::{ergo_box::*, Error};

use crate::{ErrorPtr, ReturnNum};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::delete_ptr;

// `BoxId` bindings --------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_from_str(
    box_id_str: *const c_char,
    box_id_out: *mut BoxIdPtr,
) -> ErrorPtr {
    let box_id_str = CStr::from_ptr(box_id_str).to_string_lossy();
    let res = box_id_from_str(&box_id_str, box_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_to_str(
    box_id_ptr: ConstBoxIdPtr,
    _box_id_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match box_id_to_str(box_id_ptr) {
        Ok(s) => {
            *_box_id_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_to_bytes(
    box_id_ptr: ConstBoxIdPtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = box_id_to_bytes(box_id_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_box_id_delete(ptr: BoxIdPtr) {
    unsafe { delete_ptr(ptr) }
}

// `BoxValue` bindings ------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_value_from_i64(
    amount: i64,
    box_value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = box_value_from_i64(amount, box_value_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_value_as_i64(
    box_value_ptr: ConstBoxValuePtr,
) -> ReturnNum<i64> {
    match box_value_as_i64(box_value_ptr) {
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
pub extern "C" fn ergo_wallet_box_value_delete(ptr: BoxValuePtr) {
    unsafe { delete_ptr(ptr) }
}
