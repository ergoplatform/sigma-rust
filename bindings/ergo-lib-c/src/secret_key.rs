use ergo_lib_c_core::{secret_key::*, Error};
use paste::paste;

use crate::ErrorPtr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_secret_key_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    secret_key_out: *mut SecretKeyPtr,
) -> ErrorPtr {
    let res = secret_key_from_bytes(bytes_ptr, len, secret_key_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_secret_key_generate_random(secret_key_out: *mut SecretKeyPtr) {
    #[allow(clippy::unwrap_used)]
    secret_key_generate_random(secret_key_out).unwrap();
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
pub extern "C" fn ergo_wallet_secret_key_delete(secret_key_ptr: SecretKeyPtr) {
    secret_key_delete(secret_key_ptr)
}

make_collection!(SecretKeys, SecretKey);
