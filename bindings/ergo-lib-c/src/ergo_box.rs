//! Box (aka coin, or an unspent output) is a basic concept of a UTXO-based cryptocurrency.
//! In Bitcoin, such an object is associated with some monetary value (arbitrary,
//! but with predefined precision, so we use integer arithmetic to work with the value),
//! and also a guarding script (aka proposition) to protect the box from unauthorized opening.
//!
//! In other way, a box is a state element locked by some proposition (ErgoTree).
//!
//! In Ergo, box is just a collection of registers, some with mandatory types and semantics,
//! others could be used by applications in any way.
//! We add additional fields in addition to amount and proposition~(which stored in the registers R0 and R1).
//! Namely, register R2 contains additional tokens (a sequence of pairs (token identifier, value)).
//! Register R3 contains height when block got included into the blockchain and also transaction
//! identifier and box index in the transaction outputs.
//! Registers R4-R9 are free for arbitrary usage.
//!
//! A transaction is unsealing a box. As a box can not be open twice, any further valid transaction
//! can not be linked to the same box.
use ergo_lib_c_core::{
    constant::ConstantPtr,
    contract::ConstContractPtr,
    ergo_box::*,
    ergo_tree::ErgoTreePtr,
    token::{ConstTokensPtr, TokensPtr},
    transaction::ConstTxIdPtr,
    Error,
};
use paste::paste;

use crate::{ErrorPtr, ReturnOption};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::delete_ptr;

// `BoxId` bindings --------------------------------------------------------------------------------

/// Parse box id (32 byte digest) from base16-encoded string
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_id_from_str(
    box_id_str: *const c_char,
    box_id_out: *mut BoxIdPtr,
) -> ErrorPtr {
    let box_id_str = CStr::from_ptr(box_id_str).to_string_lossy();
    let res = box_id_from_str(&box_id_str, box_id_out);
    Error::c_api_from(res)
}

/// Base16 encoded string
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_id_to_str(
    box_id_ptr: ConstBoxIdPtr,
    _box_id_str: *mut *const c_char,
) {
    #[allow(clippy::unwrap_used)]
    {
        let s = box_id_to_str(box_id_ptr).unwrap();
        *_box_id_str = CString::new(s).unwrap().into_raw();
    }
}

/// Returns byte array (32 bytes) Note: it's imperative that `output` points to a valid block of
/// memory of 32 bytes.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_id_to_bytes(box_id_ptr: ConstBoxIdPtr, output: *mut u8) {
    #[allow(clippy::unwrap_used)]
    box_id_to_bytes(box_id_ptr, output).unwrap();
}

/// Drop `BoxId`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_id_delete(ptr: BoxIdPtr) {
    delete_ptr(ptr)
}

make_ffi_eq!(BoxId);

// `BoxValue` bindings ------------------------------------------------------------------------------

/// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
/// Allows box size upto 2777 bytes with current min box value per byte of 360 nanoERGs
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_value_safe_user_min(box_value_out: *mut BoxValuePtr) {
    #[allow(clippy::unwrap_used)]
    box_value_safe_user_min(box_value_out).unwrap();
}

/// Number of units inside one ERGO (i.e. one ERG using nano ERG representation)
#[no_mangle]
pub extern "C" fn ergo_lib_box_value_units_per_ergo() -> i64 {
    box_value_units_per_ergo()
}

/// Create from i64 with bounds check
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_value_from_i64(
    amount: i64,
    box_value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = box_value_from_i64(amount, box_value_out);
    Error::c_api_from(res)
}

/// Get value as signed 64-bit long
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_value_as_i64(box_value_ptr: ConstBoxValuePtr) -> i64 {
    #[allow(clippy::unwrap_used)]
    box_value_as_i64(box_value_ptr).unwrap()
}

/// Create a new box value which is the sum of the arguments, with bounds check.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_value_sum_of(
    box_value0_ptr: ConstBoxValuePtr,
    box_value1_ptr: ConstBoxValuePtr,
    sum_of_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = box_value_sum_of(box_value0_ptr, box_value1_ptr, sum_of_out);
    Error::c_api_from(res)
}

/// Drop `BoxValue`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_box_value_delete(ptr: BoxValuePtr) {
    delete_ptr(ptr)
}

make_ffi_eq!(BoxValue);

// `ErgoBoxCandidate` bindings ----------------------------------------------------------------------

/// Returns value (ErgoTree constant) stored in the register or None if the register is empty
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_register_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> ReturnOption {
    match ergo_box_candidate_register_value(ergo_box_candidate_ptr, register_id, constant_out) {
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

/// Get box creation height
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_creation_height(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
) -> u32 {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_creation_height(ergo_box_candidate_ptr).unwrap()
}

/// Get tokens for box
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_tokens(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    tokens_out: *mut TokensPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_tokens(ergo_box_candidate_ptr, tokens_out).unwrap();
}

/// Get ergo tree for box
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_ergo_tree(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    ergo_tree_out: *mut ErgoTreePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_ergo_tree(ergo_box_candidate_ptr, ergo_tree_out).unwrap();
}

/// Get box value in nanoERGs
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_box_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    box_value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_box_value(ergo_box_candidate_ptr, box_value_out).unwrap();
}

/// Drop `ErgoBoxCandidate`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_candidate_delete(ptr: ErgoBoxCandidatePtr) {
    delete_ptr(ptr)
}

make_collection!(ErgoBoxCandidates, ErgoBoxCandidate);
make_ffi_eq!(ErgoBoxCandidate);

// `ErgoBox` bindings ------------------------------------------------------------------------------

/// Make a new box with:
/// `value` - amount of money associated with the box
/// `contract` - guarding contract([`Contract`]), which should be evaluated to true in order
/// to open(spend) this box
/// `creation_height` - height when a transaction containing the box is created.
/// `tx_id` - transaction id in which this box was "created" (participated in outputs)
/// `index` - index (in outputs) in the transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_new(
    value_ptr: ConstBoxValuePtr,
    creation_height: u32,
    contract_ptr: ConstContractPtr,
    tx_id_ptr: ConstTxIdPtr,
    index: u16,
    tokens_ptr: ConstTokensPtr,
    ergo_box_out: *mut ErgoBoxPtr,
) -> ErrorPtr {
    let res = ergo_box_new(
        value_ptr,
        creation_height,
        contract_ptr,
        tx_id_ptr,
        index,
        tokens_ptr,
        ergo_box_out,
    );
    Error::c_api_from(res)
}

/// Get box id
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_id(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_id_out: *mut BoxIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_box_id(ergo_box_ptr, box_id_out).unwrap();
}

/// Get box creation height
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_creation_height(ergo_box_ptr: ConstErgoBoxPtr) -> u32 {
    #[allow(clippy::unwrap_used)]
    ergo_box_creation_height(ergo_box_ptr).unwrap()
}

/// Get tokens for box
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_tokens(
    ergo_box_ptr: ConstErgoBoxPtr,
    tokens_out: *mut TokensPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_tokens(ergo_box_ptr, tokens_out).unwrap();
}

/// Get ergo tree for box
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_ergo_tree(
    ergo_box_ptr: ConstErgoBoxPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_ergo_tree(ergo_box_ptr, ergo_tree_out).unwrap();
}

/// Get box value in nanoERGs
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_value(ergo_box_ptr, box_value_out).unwrap();
}

/// Returns value (ErgoTree constant) stored in the register or None if the register is empty
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_register_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> ReturnOption {
    match ergo_box_register_value(ergo_box_ptr, register_id, constant_out) {
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

/// Parse from JSON.  Supports Ergo Node/Explorer API and box values and token amount encoded as
/// strings
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_from_json(
    json_str: *const c_char,
    ergo_box_out: *mut ErgoBoxPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = ergo_box_from_json(&json, ergo_box_out);
    Error::c_api_from(res)
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_to_json(
    ergo_box_ptr: ConstErgoBoxPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match ergo_box_to_json(ergo_box_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_to_json_eip12(
    ergo_box_ptr: ConstErgoBoxPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match ergo_box_to_json_eip12(ergo_box_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// Drop `ErgoBox`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_delete(ptr: ErgoBoxPtr) {
    delete_ptr(ptr)
}

make_collection!(ErgoBoxes, ErgoBox);
make_ffi_eq!(ErgoBox);

// `ErgoBoxAssetsData` bindings ---------------------------------------------------------------------

/// Create new instance
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_assets_data_new(
    value_ptr: ConstBoxValuePtr,
    tokens_ptr: ConstTokensPtr,
    ergo_box_assets_data_out: *mut ErgoBoxAssetsDataPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_assets_data_new(value_ptr, tokens_ptr, ergo_box_assets_data_out).unwrap();
}

/// Value part of the box
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_assets_data_value(
    ergo_box_assets_data_ptr: ConstErgoBoxAssetsDataPtr,
    value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_assets_data_value(ergo_box_assets_data_ptr, value_out).unwrap();
}

/// Tokens part of the box
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_assets_data_tokens(
    ergo_box_assets_data_ptr: ConstErgoBoxAssetsDataPtr,
    tokens_out: *mut TokensPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_assets_data_tokens(ergo_box_assets_data_ptr, tokens_out).unwrap();
}

/// Drop `ErgoBoxAssetsData`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_box_assets_data_delete(ptr: ErgoBoxAssetsDataPtr) {
    delete_ptr(ptr)
}

make_collection!(ErgoBoxAssetsDataList, ErgoBoxAssetsData);
make_ffi_eq!(ErgoBoxAssetsData);
