use ergo_lib_c_core::{context_extension::*, Error};

use crate::{delete_ptr, ErrorPtr, ReturnNum};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_context_extension_empty(
    context_extension_out: *mut ContextExtensionPtr,
) -> ErrorPtr {
    let res = context_extension_empty(context_extension_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_context_extension_len(
    context_extension_ptr: ConstContextExtensionPtr,
) -> ReturnNum<usize> {
    match context_extension_len(context_extension_ptr) {
        Ok(value) => crate::ReturnNum {
            value: value as usize,
            error: std::ptr::null_mut(),
        },
        Err(e) => crate::ReturnNum {
            value: 0, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_context_extension_keys(
    context_extension_ptr: ConstContextExtensionPtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = context_extension_keys(context_extension_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_context_extension_delete(ptr: ContextExtensionPtr) {
    unsafe { delete_ptr(ptr) }
}
