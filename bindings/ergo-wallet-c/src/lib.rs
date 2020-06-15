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

use std::os::raw::c_char;

// TODO: setup Xcode project and the build pipeline on CI
// TODO: share code with future JNI bindings

mod error;
pub use error::*;

pub struct Address(Box<dyn chain::Address>);

pub type ErgoStateContextPtr = *mut ergo_wallet::ErgoStateContext;
// TODO wrap enum(TBD) into struct
// TODO: make the same new/free functions as for UnspentInputBoxes
pub type AddressPtr = *mut Address;
// TODO: make the same new/free functions as for UnspentInputBoxes
pub type SecretKeyPtr = *mut ergo_wallet::SecretKey;

// TODO: sync changes to WASM API
// TODO: add docs
// TODO: extract into files/modules?

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

pub struct UnspentInputBoxes(Vec<chain::ErgoBoxCandidate>);
pub type UnspentInputBoxesPtr = *mut UnspentInputBoxes;

#[no_mangle]
pub extern "C" fn ergo_wallet_unspent_input_boxes_from_json(
    _json_str: *const u8,
    _json_str_len: usize,
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

