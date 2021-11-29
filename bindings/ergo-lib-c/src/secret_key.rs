use ergo_lib_c_core::{address::AddressPtr, secret_key::*, Error};
use paste::paste;

use crate::{delete_ptr, ErrorPtr};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_secret_key_from_bytes(
    bytes_ptr: *const u8,
    secret_key_out: *mut SecretKeyPtr,
) -> ErrorPtr {
    let res = secret_key_from_bytes(bytes_ptr, secret_key_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_secret_key_generate_random(secret_key_out: *mut SecretKeyPtr) {
    #[allow(clippy::unwrap_used)]
    secret_key_generate_random(secret_key_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_secret_key_get_address(
    secret_key_ptr: ConstSecretKeyPtr,
    address_out: *mut AddressPtr,
) {
    #[allow(clippy::unwrap_used)]
    secret_key_get_address(secret_key_ptr, address_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_secret_key_to_bytes(
    secret_key_ptr: ConstSecretKeyPtr,
    output: *mut u8,
) {
    #[allow(clippy::unwrap_used)]
    secret_key_to_bytes(secret_key_ptr, output).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_secret_key_delete(ptr: SecretKeyPtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(SecretKeys, SecretKey);
