//! Ergo input
use ergo_lib_c_core::{
    context_extension::ContextExtensionPtr, ergo_box::BoxIdPtr, input::*, Error,
};

use crate::{delete_ptr, ErrorPtr};
use paste::paste;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_input_box_id(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    box_id_out: *mut BoxIdPtr,
) -> ErrorPtr {
    let res = unsigned_input_box_id(unsigned_input_ptr, box_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_input_context_extension(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    context_extension_out: *mut ContextExtensionPtr,
) -> ErrorPtr {
    let res = unsigned_input_context_extension(unsigned_input_ptr, context_extension_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_unsigned_input_delete(ptr: UnsignedInputPtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(UnsignedInputs, UnsignedInput);
