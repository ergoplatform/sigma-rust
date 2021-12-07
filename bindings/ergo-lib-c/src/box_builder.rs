//! `ErgoBoxCandidate` builder

use ergo_lib_c_core::{
    box_builder::*,
    constant::{ConstConstantPtr, ConstantPtr},
    contract::ConstContractPtr,
    ergo_box::{BoxValuePtr, ConstBoxValuePtr, ErgoBoxCandidatePtr, NonMandatoryRegisterId},
    token::{ConstTokenAmountPtr, ConstTokenIdPtr, ConstTokenPtr},
    Error,
};
use std::{ffi::CStr, os::raw::c_char};

use crate::{delete_ptr, ErrorPtr, ReturnNum, ReturnOption};

/// Create builder with required box parameters:
/// `value` - amount of money associated with the box
/// `contract` - guarding contract([`Contract`]), which should be evaluated to true in order
/// to open(spend) this box
/// `creation_height` - height when a transaction containing the box is created.
/// It should not exceed height of the block, containing the transaction with this box.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_new(
    value_ptr: ConstBoxValuePtr,
    contract_ptr: ConstContractPtr,
    creation_height: u32,
    builder_out: *mut ErgoBoxCandidateBuilderPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_new(value_ptr, contract_ptr, creation_height, builder_out).unwrap()
}

/// Set minimal value (per byte of the serialized box size)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_set_min_box_value_per_byte(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    new_min_value_per_byte: u32,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_set_min_box_value_per_byte(builder_mut, new_min_value_per_byte)
        .unwrap();
}

/// Get minimal value (per byte of the serialized box size)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_min_box_value_per_byte(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
) -> u32 {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_min_box_value_per_byte(builder_ptr).unwrap()
}

/// Set new box value
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_set_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    value_ptr: ConstBoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_set_value(builder_mut, value_ptr).unwrap();
}

/// Get box value
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_value(builder_ptr, value_out).unwrap();
}

/// Calculate serialized box size(in bytes)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_calc_box_size_bytes(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
) -> ReturnNum<usize> {
    match ergo_box_candidate_builder_calc_box_size_bytes(builder_ptr) {
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

/// Calculate minimal box value for the current box serialized size(in bytes)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_calc_min_box_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = ergo_box_candidate_builder_calc_min_box_value(builder_ptr, value_out);
    Error::c_api_from(res)
}

/// Set register with a given id (R4-R9) to the given value
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_set_register_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
    constant_ptr: ConstConstantPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_set_register_value(builder_mut, register_id, constant_ptr).unwrap();
}

/// Returns register value for the given register id (R4-R9), or None if the register is empty
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_register_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> ReturnOption {
    match ergo_box_candidate_builder_register_value(builder_ptr, register_id, constant_out) {
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

/// Delete register value(make register empty) for the given register id (R4-R9)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_delete_register_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_delete_register_value(builder_mut, register_id).unwrap();
}

/// Mint token, as defined in <https://github.com/ergoplatform/eips/blob/master/eip-0004.md>
/// `token` - token id(box id of the first input box in transaction) and token amount,
/// `token_name` - token name (will be encoded in R4),
/// `token_desc` - token description (will be encoded in R5),
/// `num_decimals` - number of decimals (will be encoded in R6)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_mint_token(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    token_ptr: ConstTokenPtr,
    token_name: *const c_char,
    token_desc: *const c_char,
    num_decimals: usize,
) {
    let token_name = CStr::from_ptr(token_name).to_string_lossy();
    let token_desc = CStr::from_ptr(token_desc).to_string_lossy();
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_mint_token(
        builder_mut,
        token_ptr,
        &token_name,
        &token_desc,
        num_decimals,
    )
    .unwrap();
}

/// Add given token id and token amount
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_add_token(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    token_id_ptr: ConstTokenIdPtr,
    token_amount_ptr: ConstTokenAmountPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_builder_add_token(builder_mut, token_id_ptr, token_amount_ptr).unwrap();
}

/// Build the box candidate
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_builder_build(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    ergo_box_candidate_out: *mut ErgoBoxCandidatePtr,
) -> ErrorPtr {
    let res = ergo_box_candidate_builder_build(builder_ptr, ergo_box_candidate_out);
    Error::c_api_from(res)
}

/// Drop `ErgoBoxCandidateBuilder`
#[no_mangle]
pub extern "C" fn ergo_lib_ergo_box_candidate_builder_delete(ptr: ErgoBoxCandidateBuilderPtr) {
    unsafe { delete_ptr(ptr) }
}
