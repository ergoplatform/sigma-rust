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
mod box_selector;
mod byte_array;
mod constant;
mod context_extension;
mod contract;
mod data_input;
mod ergo_box;
mod ergo_state_ctx;
mod ergo_tree;
mod ext_secret_key;
mod header;
mod input;
mod merkleproof;
mod reduced;
mod secret_key;
mod token;
mod transaction;
mod tx_builder;
mod wallet;

pub use crate::address::*;
pub use crate::block_header::*;
pub use crate::box_builder::*;
pub use crate::box_selector::*;
pub use crate::byte_array::*;
pub use crate::context_extension::*;
pub use crate::contract::*;
pub use crate::data_input::*;
pub use crate::ergo_box::*;
pub use crate::ergo_state_ctx::*;
pub use crate::ergo_tree::*;
pub use crate::header::*;
pub use crate::input::*;
pub use crate::merkleproof::*;
pub use crate::reduced::*;
pub use crate::secret_key::*;
pub use crate::token::*;
pub use crate::transaction::*;
pub use crate::tx_builder::*;
pub use crate::wallet::*;
pub use ergo_lib_c_core::{
    address::{Address, AddressTypePrefix, NetworkPrefix},
    Error,
};
use std::{ffi::CString, os::raw::c_char};

pub type ErrorPtr = *mut Error;

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_delete_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let cstring = CString::from_raw(ptr);
        std::mem::drop(cstring)
    }
}

#[no_mangle]
pub extern "C" fn ergo_lib_delete_error(error: ErrorPtr) {
    if !error.is_null() {
        let boxed = unsafe { Box::from_raw(error) };
        std::mem::drop(boxed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_error_to_string(error: ErrorPtr) -> *mut c_char {
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
