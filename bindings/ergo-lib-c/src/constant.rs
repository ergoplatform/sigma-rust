use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use ergo_lib_c_core::{
    constant::*,
    ergo_box::{ConstErgoBoxPtr, ErgoBoxPtr},
    Error, ErrorPtr,
};
use paste::paste;

use crate::{delete_ptr, ReturnNum};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_from_base16(
    bytes_ptr: *const c_char,
    constant_out: *mut ConstantPtr,
) -> ErrorPtr {
    let bytes_str = CStr::from_ptr(bytes_ptr).to_string_lossy();
    let res = constant_from_base16_bytes(&bytes_str, constant_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_to_base16(
    constant_ptr: ConstConstantPtr,
    _bytes_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match constant_to_base16_str(constant_ptr) {
        Ok(s) => {
            *_bytes_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_from_i32(value: i32, constant_out: *mut ConstantPtr) {
    #[allow(clippy::unwrap_used)]
    constant_from_i32(value, constant_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_to_i32(
    constant_ptr: ConstConstantPtr,
) -> ReturnNum<i32> {
    match constant_to_i32(constant_ptr) {
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
pub unsafe extern "C" fn ergo_wallet_constant_from_i64(value: i64, constant_out: *mut ConstantPtr) {
    #[allow(clippy::unwrap_used)]
    constant_from_i64(value, constant_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_to_i64(
    constant_ptr: ConstConstantPtr,
) -> ReturnNum<i64> {
    match constant_to_i64(constant_ptr) {
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
pub unsafe extern "C" fn ergo_wallet_constant_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    constant_out: *mut ConstantPtr,
) -> ErrorPtr {
    let res = constant_from_bytes(bytes_ptr, len, constant_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_bytes_len(
    constant_ptr: ConstConstantPtr,
) -> ReturnNum<usize> {
    match constant_bytes_len(constant_ptr) {
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
pub unsafe extern "C" fn ergo_wallet_constant_to_bytes(
    constant_ptr: ConstConstantPtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = constant_to_bytes(constant_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_from_ecpoint_bytes(
    bytes_ptr: *const u8,
    len: usize,
    constant_out: *mut ConstantPtr,
) -> ErrorPtr {
    let res = constant_from_ecpoint_bytes(bytes_ptr, len, constant_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_from_ergo_box(
    ergo_box_ptr: ConstErgoBoxPtr,
    constant_out: *mut ConstantPtr,
) {
    #[allow(clippy::unwrap_used)]
    constant_from_ergo_box(ergo_box_ptr, constant_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_to_ergo_box(
    constant_ptr: ConstConstantPtr,
    ergo_box_out: *mut ErgoBoxPtr,
) -> ErrorPtr {
    let res = constant_to_ergo_box(constant_ptr, ergo_box_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_constant_delete(ptr: ConstantPtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(Constant);
