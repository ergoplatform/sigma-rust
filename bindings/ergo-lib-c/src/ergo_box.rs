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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_from_str(
    box_id_str: *const c_char,
    box_id_out: *mut BoxIdPtr,
) -> ErrorPtr {
    let box_id_str = CStr::from_ptr(box_id_str).to_string_lossy();
    let res = box_id_from_str(&box_id_str, box_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_to_str(
    box_id_ptr: ConstBoxIdPtr,
    _box_id_str: *mut *const c_char,
) {
    #[allow(clippy::unwrap_used)]
    {
        let s = box_id_to_str(box_id_ptr).unwrap();
        *_box_id_str = CString::new(s).unwrap().into_raw();
    }
}

/// Note: it's imperative that `output` points to a valid block of memory of 32 bytes.
#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_to_bytes(box_id_ptr: ConstBoxIdPtr, output: *mut u8) {
    #[allow(clippy::unwrap_used)]
    box_id_to_bytes(box_id_ptr, output).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_box_id_delete(ptr: BoxIdPtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(BoxId);

// `BoxValue` bindings ------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_value_safe_user_min(box_value_out: *mut BoxValuePtr) {
    #[allow(clippy::unwrap_used)]
    box_value_safe_user_min(box_value_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_box_value_units_per_ergo() -> i64 {
    box_value_units_per_ergo()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_value_from_i64(
    amount: i64,
    box_value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = box_value_from_i64(amount, box_value_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_value_as_i64(box_value_ptr: ConstBoxValuePtr) -> i64 {
    #[allow(clippy::unwrap_used)]
    box_value_as_i64(box_value_ptr).unwrap()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_box_value_delete(ptr: BoxValuePtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(BoxValue);

// `ErgoBoxCandidate` bindings ----------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_register_value(
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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_creation_height(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
) -> u32 {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_creation_height(ergo_box_candidate_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_tokens(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    tokens_out: *mut TokensPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_tokens(ergo_box_candidate_ptr, tokens_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_ergo_tree(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    ergo_tree_out: *mut ErgoTreePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_ergo_tree(ergo_box_candidate_ptr, ergo_tree_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_box_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    box_value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_candidate_box_value(ergo_box_candidate_ptr, box_value_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_box_candidate_delete(ptr: ErgoBoxCandidatePtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(ErgoBoxCandidates, ErgoBoxCandidate);
make_ffi_eq!(ErgoBoxCandidate);

// `ErgoBox` bindings ------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_new(
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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_id(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_id_out: *mut BoxIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_box_id(ergo_box_ptr, box_id_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_creation_height(
    ergo_box_ptr: ConstErgoBoxPtr,
) -> u32 {
    #[allow(clippy::unwrap_used)]
    ergo_box_creation_height(ergo_box_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_tokens(
    ergo_box_ptr: ConstErgoBoxPtr,
    tokens_out: *mut TokensPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_tokens(ergo_box_ptr, tokens_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_ergo_tree(
    ergo_box_ptr: ConstErgoBoxPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_ergo_tree(ergo_box_ptr, ergo_tree_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_value(ergo_box_ptr, box_value_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_register_value(
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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_from_json(
    json_str: *const c_char,
    ergo_box_out: *mut ErgoBoxPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = ergo_box_from_json(&json, ergo_box_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_to_json(
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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_to_json_eip12(
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

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_box_delete(ptr: ErgoBoxPtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(ErgoBoxes, ErgoBox);
make_ffi_eq!(ErgoBox);

// `ErgoBoxAssetsData` bindings ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_assets_data_new(
    value_ptr: ConstBoxValuePtr,
    tokens_ptr: ConstTokensPtr,
    ergo_box_assets_data_out: *mut ErgoBoxAssetsDataPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_assets_data_new(value_ptr, tokens_ptr, ergo_box_assets_data_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_assets_data_value(
    ergo_box_assets_data_ptr: ConstErgoBoxAssetsDataPtr,
    value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_assets_data_value(ergo_box_assets_data_ptr, value_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_assets_data_tokens(
    ergo_box_assets_data_ptr: ConstErgoBoxAssetsDataPtr,
    tokens_out: *mut TokensPtr,
) {
    #[allow(clippy::unwrap_used)]
    ergo_box_assets_data_tokens(ergo_box_assets_data_ptr, tokens_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_box_assets_data_delete(ptr: ErgoBoxAssetsDataPtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(ErgoBoxAssetsDataList, ErgoBoxAssetsData);
make_ffi_eq!(ErgoBoxAssetsData);
