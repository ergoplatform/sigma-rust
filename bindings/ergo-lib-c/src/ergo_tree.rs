//! ErgoTree

use ergo_lib_c_core::{
    constant::{ConstConstantPtr, ConstantPtr},
    ergo_tree::*,
    Error,
};
use paste::paste;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{delete_ptr, ErrorPtr, ReturnNum, ReturnOption};

/// Decode from base16 encoded serialized ErgoTree
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_from_base16_bytes(
    bytes_ptr: *const c_char,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let bytes_str = CStr::from_ptr(bytes_ptr).to_string_lossy();
    let res = ergo_tree_from_base16_bytes(&bytes_str, ergo_tree_out);
    Error::c_api_from(res)
}

/// Decode from encoded serialized ErgoTree
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let res = ergo_tree_from_bytes(bytes_ptr, len, ergo_tree_out);
    Error::c_api_from(res)
}

/// Determine number of bytes of the serialized ErgoTree
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_bytes_len(
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

/// Convert to serialized bytes. **Key assumption:** enough memory has been allocated at the address
/// pointed-to by `output`. Use `ergo_lib_ergo_tree_bytes_len` to determine the length of the
/// byte array.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_to_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = ergo_tree_to_bytes(ergo_tree_ptr, output);
    Error::c_api_from(res)
}

/// Get Base16-encoded serialized bytes
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_to_base16_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    _str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match ergo_tree_to_base16_bytes(ergo_tree_ptr) {
        Ok(s) => {
            *_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// Get constants number as stored in serialized ErgoTree or error if the parsing of
/// constants is failed
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_constants_len(
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

/// Returns constant with given index (as stored in serialized ErgoTree)
/// or None if index is out of bounds
/// or error if constants parsing were failed
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_get_constant(
    ergo_tree_ptr: ConstErgoTreePtr,
    index: usize,
    constant_out: *mut ConstantPtr,
) -> ReturnOption {
    match ergo_tree_get_constant(ergo_tree_ptr, index, constant_out) {
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

/// Returns new ErgoTree instance with a new constant value for a given index in constants list (as
/// stored in serialized ErgoTree), or an error. Note that the original ErgoTree instance
/// pointed-at by `ergo_tree_ptr` is untouched.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_with_constant(
    ergo_tree_ptr: ConstErgoTreePtr,
    index: usize,
    constant_ptr: ConstConstantPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let res = ergo_tree_with_constant(ergo_tree_ptr, index, constant_ptr, ergo_tree_out);
    Error::c_api_from(res)
}

/// Returns the number of bytes of the Serialized proposition expression of SigmaProp type with
/// ConstantPlaceholder nodes instead of Constant nodes.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_template_bytes_len(
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

/// Serialized proposition expression of SigmaProp type with ConstantPlaceholder nodes instead of
/// Constant nodes. Key assumption: enough memory has been allocated at the address pointed-to by
/// `output`. Use `ergo_lib_ergo_tree_template_bytes_len` to determine the length of the byte
/// array.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_tree_template_bytes(
    ergo_tree_ptr: ConstErgoTreePtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = ergo_tree_template_bytes(ergo_tree_ptr, output);
    Error::c_api_from(res)
}

/// Drop `ErgoTree`
#[no_mangle]
pub extern "C" fn ergo_lib_ergo_tree_delete(ptr: ErgoTreePtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(ErgoTree);
