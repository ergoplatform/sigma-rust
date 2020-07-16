//! WASM bindings for sigma-tree

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// #![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)]

use sigma_tree::chain;

use ergo_wallet_lib_c_core::{address_delete, address_from_testnet, AddressPtr, Error, ErrorPtr};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

pub struct ErgoStateContext(ergo_wallet_lib::ErgoStateContext);
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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_from_testnet(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_testnet(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_address_delete(address: AddressPtr) {
    address_delete(address)
}

pub struct UnspentBoxes(Vec<chain::ErgoBoxCandidate>);
pub type UnspentBoxesPtr = *mut UnspentBoxes;

#[no_mangle]
pub extern "C" fn ergo_wallet_unspent_boxes_from_json(
    _json_str: *const c_char,
    _unspent_boxes_out: *mut UnspentBoxesPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_unspent_boxes_delete(_unspent_boxes: UnspentBoxesPtr) -> ErrorPtr {
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

pub struct Wallet();
pub type WalletPtr = *mut Wallet;

#[no_mangle]
pub extern "C" fn ergo_wallet_wallet_from_mnemonic(
    _mnemonic_phrase: *const c_char,
    _mnemonic_password: *const u8,
    _mnemonic_password_length: usize,
    _wallet_out: *mut WalletPtr,
) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_wallet_delete(_wallet: WalletPtr) -> ErrorPtr {
    todo!()
}

#[no_mangle]
pub extern "C" fn ergo_wallet_wallet_new_signed_tx(
    _wallet: WalletPtr,
    _state_context: ErgoStateContextPtr,
    _unspent_boxes: UnspentBoxesPtr,
    _data_input_boxes: DataInputBoxesPtr, // can be null
    _output_boxes: OutputBoxesPtr,
    _send_change_to: AddressPtr,
    _min_change_value: u64,
    _tx_fee_amount: u64,
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
