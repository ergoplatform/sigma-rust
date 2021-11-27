//! C bindings for ergo-lib

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// #![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
mod macros;
mod address;
mod block_header;
mod box_builder;
mod byte_array;
mod constant;
mod context_extension;
mod contract;
mod data_input;
mod ergo_box;
mod ergo_state_ctx;
mod ergo_tree;
mod header;
mod input;
mod secret_key;
mod token;
mod transaction;
use ergo_lib::ergotree_ir::chain;

pub use crate::address::*;
pub use crate::block_header::*;
pub use crate::box_builder::*;
pub use crate::byte_array::*;
pub use crate::context_extension::*;
pub use crate::contract::*;
pub use crate::data_input::*;
pub use crate::ergo_box::*;
pub use crate::ergo_state_ctx::*;
pub use crate::ergo_tree::*;
pub use crate::header::*;
pub use crate::input::*;
pub use crate::secret_key::*;
pub use crate::token::*;
pub use crate::transaction::*;
pub use ergo_lib_c_core::{
    address::{Address, AddressTypePrefix, NetworkPrefix},
    Error,
};
use std::{ffi::CString, os::raw::c_char};

pub type ErrorPtr = *mut Error;

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
    #[allow(clippy::unwrap_used)]
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

/// Convenience type to allow us to pass Rust `Option<_>` types through FFI to C side.
#[repr(C)]
pub struct ReturnOption {
    is_some: bool,
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
impl IntegerType for i32 {}
impl IntegerType for u32 {}
impl IntegerType for i64 {}
impl IntegerType for usize {}
