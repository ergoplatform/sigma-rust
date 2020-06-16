//! WASM bindings for sigma-tree

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// #![deny(missing_docs)]

use sigma_tree::chain;

use std::{ffi::CString, os::raw::c_char};

// TODO: sync changes to WASM API
// TODO: share code with future JNI bindings
// TODO: add docs
// TODO: extract into files/modules?

mod error;
pub use error::*;

pub struct ErgoStateContext(ergo_wallet::ErgoStateContext);
pub type ErgoStateContextPtr = *mut ErgoStateContext;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_state_context_from_json(
    _json_str: *const c_char,
    _ergo_state_context_out: *mut ErgoStateContextPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_state_context_delete(
    _ergo_state_context: ErgoStateContextPtr,
) -> ErrorPtr {
    todo!()
}

pub struct SecretKey(ergo_wallet::SecretKey);
pub type SecretKeyPtr = *mut SecretKey;

#[no_mangle]
pub extern "C" fn ergo_wallet_secret_key_parse_str(
    _secret_key_str: *const c_char,
    _secret_key_out: *mut SecretKeyPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_secret_key_delete(_secret_key: SecretKeyPtr) -> ErrorPtr {
    todo!()
}

pub struct ErgoBoxCandidate(chain::ErgoBoxCandidate);
pub type ErgoBoxCandidatePtr = *mut ErgoBoxCandidate;

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_box_candidate_new_pay_to_address(
    _recipient: AddressPtr,
    _value: u64,
    _creation_height: u32,
    _ergo_box_candidate_out: *mut ErgoBoxCandidatePtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_ergo_box_candidate_delete(
    _ergo_box_candidate: ErgoBoxCandidatePtr,
) -> ErrorPtr {
    todo!()
}

pub struct Address(Box<dyn chain::Address>);
pub type AddressPtr = *mut Address;

#[no_mangle]
pub extern "C" fn ergo_wallet_address_from_testnet(
    _address_str: *const c_char,
    _address_out: *mut AddressPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_address_delete(_address: AddressPtr) -> ErrorPtr {
    todo!()
}

pub struct UnspentInputBoxes(Vec<chain::ErgoBoxCandidate>);
pub type UnspentInputBoxesPtr = *mut UnspentInputBoxes;

#[no_mangle]
pub extern "C" fn ergo_wallet_unspent_input_boxes_from_json(
    _json_str: *const c_char,
    _unspent_input_boxes_out: *mut UnspentInputBoxesPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_unspent_input_boxes_delete(
    _unspent_input_boxes: UnspentInputBoxesPtr,
) -> ErrorPtr {
    todo!()
}

pub struct DataInputBoxes(Vec<chain::ErgoBoxCandidate>);
pub type DataInputBoxesPtr = *mut DataInputBoxes;

pub struct OutputBoxes(Vec<chain::ErgoBoxCandidate>);
pub type OutputBoxesPtr = *mut OutputBoxes;

#[no_mangle]
pub extern "C" fn ergo_wallet_output_boxes_new(
    _ergo_box_candidate: ErgoBoxCandidatePtr,
    _output_boxes_out: *mut OutputBoxesPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_output_boxes_delete(_output_boxes: OutputBoxesPtr) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_new_signed_tx(
    _state_context: ErgoStateContextPtr, // can be null or make "empty" func?
    _unspent_input_boxes: UnspentInputBoxesPtr,
    _data_input_boxes: DataInputBoxesPtr, // can be null
    _output_boxes: OutputBoxesPtr,
    _send_change_to: AddressPtr,
    _min_change_value: u64,
    _tx_fee_amount: u64,
    _sk: SecretKeyPtr,
    _transaction_out: *mut TransactionPtr,
) -> ErrorPtr {
    todo!()
}

pub struct Transaction(chain::Transaction);
pub type TransactionPtr = *mut Transaction;

#[no_mangle]
pub extern "C" fn ergo_wallet_delete_signed_tx(_transaction: TransactionPtr) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_signed_tx_to_json(
    _transaction: TransactionPtr,
    _json_str_out: *mut *const c_char,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_delete_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let cstring = CString::from_raw(ptr);
        std::mem::drop(cstring)
    }
}

#[no_mangle]
pub extern "C" fn ergo_wallet_delete_error(error: ErrorPtr) {
    if !error.is_null() {
        let boxed = unsafe { Box::from_raw(error) };
        std::mem::drop(boxed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_error_to_string(error: ErrorPtr) -> *mut c_char {
    if let Some(error) = error.as_ref() {
        CString::new(error.to_string()).unwrap().into_raw()
    } else {
        CString::new(b"success".to_vec()).unwrap().into_raw()
    }
}
