use ergo_lib_c_core::context_extension::*;

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_context_extension_empty(
    context_extension_out: *mut ContextExtensionPtr,
) {
    #[allow(clippy::unwrap_used)]
    context_extension_empty(context_extension_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_context_extension_len(
    context_extension_ptr: ConstContextExtensionPtr,
) -> usize {
    #[allow(clippy::unwrap_used)]
    context_extension_len(context_extension_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_context_extension_keys(
    context_extension_ptr: ConstContextExtensionPtr,
    output: *mut u8,
) {
    #[allow(clippy::unwrap_used)]
    context_extension_keys(context_extension_ptr, output).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_context_extension_delete(ptr: ContextExtensionPtr) {
    unsafe { delete_ptr(ptr) }
}
