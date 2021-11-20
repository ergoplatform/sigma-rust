//! C bindings for ergo-lib

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// #![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
mod macros;
mod address;
mod block_header;
mod constant;
mod context_extension;
mod data_input;
mod ergo_box;
mod ergo_state_ctx;
mod ergo_tree;
mod header;
mod input;
mod secret_key;
mod transaction;
use ergo_lib::ergotree_ir::chain;

pub use crate::address::*;
pub use crate::block_header::*;
pub use crate::context_extension::*;
pub use crate::data_input::*;
pub use crate::ergo_box::*;
pub use crate::ergo_state_ctx::*;
pub use crate::ergo_tree::*;
pub use crate::header::*;
pub use crate::input::*;
pub use crate::secret_key::*;
pub use crate::transaction::*;
use ergo_lib_c_core::{address::AddressPtr, ergo_state_ctx::ErgoStateContextPtr};
pub use ergo_lib_c_core::{
    address::{Address, AddressTypePrefix, NetworkPrefix},
    Error,
};
use std::{ffi::CString, os::raw::c_char};

pub type ErrorPtr = *mut Error;

pub struct ErgoBoxCandidate(chain::ergo_box::ErgoBoxCandidate);
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

pub struct UnspentBoxes(Vec<chain::ergo_box::ErgoBoxCandidate>);
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

pub struct DataInputBoxes(Vec<chain::ergo_box::ErgoBoxCandidate>);
pub type DataInputBoxesPtr = *mut DataInputBoxes;

pub struct OutputBoxes(Vec<chain::ergo_box::ErgoBoxCandidate>);
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

pub struct Transaction(ergo_lib::chain::transaction::Transaction);
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

/// Convenience type to allow us to pass Rust enums with `u8` representation through FFI to the C
/// side.
#[repr(C)]
pub struct ReturnNum<T: IntegerType> {
    /// Returned value. Note that it's only valid if the error field is null!
    value: T,
    error: ErrorPtr,
}

/// Convenience type to allow us to pass Rust `Option<T>` through FFI to C side.
#[repr(C)]
pub struct ReturnOption<T> {
    is_some: bool,
    value_ptr: *mut *mut T,
    error: ErrorPtr,
}

pub unsafe fn delete_ptr<T>(ptr: *mut T) {
    if !ptr.is_null() {
        let boxed = Box::from_raw(ptr);
        std::mem::drop(boxed);
    }
}
pub trait IntegerType {}

impl IntegerType for u8 {}
impl IntegerType for usize {}
