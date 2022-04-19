//! Ergo input
use std::{ffi::CString, os::raw::c_char};

use ergo_lib_c_core::{
    context_extension::ContextExtensionPtr, ergo_box::BoxIdPtr, input::*, Error,
};

use crate::{delete_ptr, ErrorPtr};
use paste::paste;

// `UnsignedInput` bindings -------------------------------------------------------------------------

/// Get box id
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_input_box_id(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    box_id_out: *mut BoxIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    unsigned_input_box_id(unsigned_input_ptr, box_id_out).unwrap();
}

/// Get extension
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_input_context_extension(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    context_extension_out: *mut ContextExtensionPtr,
) {
    #[allow(clippy::unwrap_used)]
    unsigned_input_context_extension(unsigned_input_ptr, context_extension_out).unwrap();
}

/// Drop `UnsignedInput`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_input_delete(ptr: UnsignedInputPtr) {
    delete_ptr(ptr)
}

make_collection!(UnsignedInputs, UnsignedInput);

// `Input` bindings --------------------------------------------------------------------------------

/// Get box id
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_input_box_id(
    input_ptr: ConstInputPtr,
    box_id_out: *mut BoxIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    input_box_id(input_ptr, box_id_out).unwrap();
}

/// Get spending proof
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_input_spending_proof(
    input_ptr: ConstInputPtr,
    prover_result_out: *mut ProverResultPtr,
) {
    #[allow(clippy::unwrap_used)]
    input_spending_proof(input_ptr, prover_result_out).unwrap();
}

/// Drop `Input`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_input_delete(ptr: InputPtr) {
    delete_ptr(ptr)
}

make_collection!(Inputs, Input);

// `ProverResult` bindings -------------------------------------------------------------------------

/// Return the number of bytes that make up the proof.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_prover_result_proof_len(
    prover_result_ptr: ConstProverResultPtr,
) -> usize {
    #[allow(clippy::unwrap_used)]
    prover_result_proof_len(prover_result_ptr).unwrap()
}

/// Get proof. Key assumption: enough memory has been allocated at the address pointed-to by
/// `output`. Use `ergo_lib_prover_result_proof_len` to determine the length of the byte
/// array.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_prover_result_proof(
    prover_result_ptr: ConstProverResultPtr,
    output: *mut u8,
) {
    #[allow(clippy::unwrap_used)]
    prover_result_proof(prover_result_ptr, output).unwrap();
}

/// Get extension
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_prover_result_context_extension(
    prover_result_ptr: ConstProverResultPtr,
    context_extension_out: *mut ContextExtensionPtr,
) {
    #[allow(clippy::unwrap_used)]
    prover_result_context_extension(prover_result_ptr, context_extension_out).unwrap();
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_prover_result_to_json(
    prover_result_ptr: ConstProverResultPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match prover_result_to_json(prover_result_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// Drop `ProverResult`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_prover_result_delete(ptr: ProverResultPtr) {
    delete_ptr(ptr)
}
