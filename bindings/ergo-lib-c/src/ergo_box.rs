use ergo_lib_c_core::{
    constant::{Constant, ConstantPtr},
    contract::ConstContractPtr,
    ergo_box::*,
    ergo_tree::ErgoTreePtr,
    token::{ConstTokensPtr, TokensPtr},
    transaction::ConstTxIdPtr,
    Error,
};
use paste::paste;

use crate::{ErrorPtr, ReturnNum, ReturnOption};
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
) -> ErrorPtr {
    let res = match box_id_to_str(box_id_ptr) {
        Ok(s) => {
            *_box_id_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_id_to_bytes(
    box_id_ptr: ConstBoxIdPtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = box_id_to_bytes(box_id_ptr, output);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_box_id_delete(ptr: BoxIdPtr) {
    unsafe { delete_ptr(ptr) }
}

// `BoxValue` bindings ------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_value_safe_user_min(
    box_value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = box_value_safe_user_min(box_value_out);
    Error::c_api_from(res)
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
pub unsafe extern "C" fn ergo_wallet_box_value_as_i64(
    box_value_ptr: ConstBoxValuePtr,
) -> ReturnNum<i64> {
    match box_value_as_i64(box_value_ptr) {
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
pub extern "C" fn ergo_wallet_box_value_delete(ptr: BoxValuePtr) {
    unsafe { delete_ptr(ptr) }
}

// `ErgoBoxCandidate` bindings ----------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_register_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> ReturnOption<Constant> {
    match ergo_box_candidate_register_value(ergo_box_candidate_ptr, register_id, constant_out) {
        Ok(is_some) => ReturnOption {
            is_some,
            value_ptr: constant_out,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnOption {
            is_some: false, // Just a dummy value
            value_ptr: constant_out,
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_creation_height(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
) -> ReturnNum<u32> {
    match ergo_box_candidate_creation_height(ergo_box_candidate_ptr) {
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
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_tokens(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    tokens_out: *mut TokensPtr,
) -> ErrorPtr {
    let res = ergo_box_candidate_tokens(ergo_box_candidate_ptr, tokens_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_ergo_tree(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let res = ergo_box_candidate_ergo_tree(ergo_box_candidate_ptr, ergo_tree_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_candidate_box_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    box_value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = ergo_box_candidate_box_value(ergo_box_candidate_ptr, box_value_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_box_candidate_delete(ptr: ErgoBoxCandidatePtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(ErgoBoxCandidates, ErgoBoxCandidate);

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
) -> ErrorPtr {
    let res = ergo_box_box_id(ergo_box_ptr, box_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_creation_height(
    ergo_box_ptr: ConstErgoBoxPtr,
) -> ReturnNum<u32> {
    match ergo_box_creation_height(ergo_box_ptr) {
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
pub unsafe extern "C" fn ergo_wallet_ergo_box_tokens(
    ergo_box_ptr: ConstErgoBoxPtr,
    tokens_out: *mut TokensPtr,
) -> ErrorPtr {
    let res = ergo_box_tokens(ergo_box_ptr, tokens_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_ergo_tree(
    ergo_box_ptr: ConstErgoBoxPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> ErrorPtr {
    let res = ergo_box_ergo_tree(ergo_box_ptr, ergo_tree_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_value_out: *mut BoxValuePtr,
) -> ErrorPtr {
    let res = ergo_box_value(ergo_box_ptr, box_value_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_box_register_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> ReturnOption<Constant> {
    match ergo_box_register_value(ergo_box_ptr, register_id, constant_out) {
        Ok(is_some) => ReturnOption {
            is_some,
            value_ptr: constant_out,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnOption {
            is_some: false, // Just a dummy value
            value_ptr: constant_out,
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
