//! Extended Secret Key functionality

use std::{ffi::CStr, os::raw::c_char};

use ergo_lib_c_core::{
    ext_secret_key::{
        ext_secret_key_derive, ext_secret_key_derive_master, ext_secret_key_new,
        ConstExtSecretKeyPtr, ExtSecretKeyPtr,
    },
    Error,
};

use crate::{delete_ptr, ErrorPtr};

/// Create ExtSecretKey from secret key bytes, chain code and derivation path
/// Derivation path should be a string in the form of: m/44/429/acc'/0/addr
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ext_secret_key_new(
    secret_key_bytes_ptr: *const u8,
    chain_code_ptr: *const u8,
    derivation_path_str: *const c_char,
    ext_secret_key_out: *mut ExtSecretKeyPtr,
) -> ErrorPtr {
    let derivation_path = CStr::from_ptr(derivation_path_str).to_string_lossy();
    let res = ext_secret_key_new(
        secret_key_bytes_ptr,
        chain_code_ptr,
        &derivation_path,
        ext_secret_key_out,
    );
    Error::c_api_from(res)
}

/// Derive root extended secret key from seed bytes
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ext_secret_key_derive_master(
    seed: *const u8,
    ext_secret_key_out: *mut ExtSecretKeyPtr,
) -> ErrorPtr {
    let res = ext_secret_key_derive_master(seed, ext_secret_key_out);
    Error::c_api_from(res)
}

/// Derive a new extended secret key from the provided index
/// The index is in the form of soft or hardened indices
/// For example: 4 or 4' respectively
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ext_secret_key_derive(
    secret_key_bytes_ptr: ConstExtSecretKeyPtr,
    index_str: *const c_char,
    ext_secret_key_out: *mut ExtSecretKeyPtr,
) -> ErrorPtr {
    let index = CStr::from_ptr(index_str).to_string_lossy();
    let res = ext_secret_key_derive(secret_key_bytes_ptr, &index, ext_secret_key_out);
    Error::c_api_from(res)
}

/// Drop `ExtSecretKey`
#[no_mangle]
pub extern "C" fn ergo_lib_ext_secret_key_delete(ptr: ExtSecretKeyPtr) {
    unsafe { delete_ptr(ptr) }
}
