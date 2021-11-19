use ergo_lib_c_core::{ergo_box::*, Error};

use crate::ErrorPtr;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::delete_ptr;

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
