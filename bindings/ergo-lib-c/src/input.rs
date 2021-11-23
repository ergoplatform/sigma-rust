//! Ergo input
use std::{ffi::CString, os::raw::c_char};

use ergo_lib_c_core::{
    context_extension::ContextExtensionPtr, ergo_box::BoxIdPtr, input::*, Error,
};

use crate::{delete_ptr, ErrorPtr, ReturnNum};
use paste::paste;

// `UnsignedInput` bindings -------------------------------------------------------------------------

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

// `Input` bindings --------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_input_box_id(
    input_ptr: ConstInputPtr,
    box_id_out: *mut BoxIdPtr,
) -> ErrorPtr {
    let res = input_box_id(input_ptr, box_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_input_spending_proof(
    input_ptr: ConstInputPtr,
    prover_result_out: *mut ProverResultPtr,
) -> ErrorPtr {
    let res = input_spending_proof(input_ptr, prover_result_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_input_delete(ptr: InputPtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(Inputs, Input);

// `ProverResult` bindings -------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_prover_result_proof_len(
    prover_result_ptr: ConstProverResultPtr,
) -> ReturnNum<usize> {
    match prover_result_proof_len(prover_result_ptr) {
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
pub unsafe extern "C" fn ergo_wallet_prover_result_proof(
    prover_result_ptr: ConstProverResultPtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = prover_result_proof(prover_result_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_prover_result_context_extension(
    prover_result_ptr: ConstProverResultPtr,
    context_extension_out: *mut ContextExtensionPtr,
) -> ErrorPtr {
    let res = prover_result_context_extension(prover_result_ptr, context_extension_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_prover_result_to_json(
    prover_result_ptr: ConstProverResultPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match prover_result_to_json(prover_result_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_prover_result_delete(ptr: ProverResultPtr) {
    unsafe { delete_ptr(ptr) }
}
