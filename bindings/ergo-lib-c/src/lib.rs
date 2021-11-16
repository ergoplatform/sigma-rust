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
mod secret_key;
use ergo_lib::ergotree_ir::chain;
use paste::paste;

pub use crate::secret_key::*;
use ergo_lib_c_core::{
    address::{
        address_delete, address_from_base58, address_from_mainnet, address_from_testnet,
        address_to_base58, address_type_prefix, AddressPtr, ConstAddressPtr,
    },
    block_header::{
        block_header_delete, block_header_from_json, BlockHeader, BlockHeaderPtr,
        ConstBlockHeaderPtr,
    },
    collections::{
        collection_add, collection_delete, collection_get, collection_len, collection_new,
        CollectionPtr, ConstCollectionPtr,
    },
    ergo_state_ctx::{ergo_state_context_delete, ergo_state_context_new, ErgoStateContextPtr},
    header::{preheader_delete, preheader_from_block_header, ConstPreHeaderPtr, PreHeaderPtr},
};
pub use ergo_lib_c_core::{
    address::{Address, AddressTypePrefix, NetworkPrefix},
    Error,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

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

// -------------------------------------------------------------------------------------------------
// Address functions -------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

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
pub unsafe extern "C" fn ergo_wallet_address_from_mainnet(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_mainnet(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_from_base58(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_base58(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_to_base58(
    address: ConstAddressPtr,
    network_prefix: NetworkPrefix,
    _address_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match address_to_base58(address, network_prefix) {
        Ok(s) => {
            *_address_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_type_prefix(
    address: ConstAddressPtr,
) -> ReturnNum<u8> {
    match address_type_prefix(address) {
        Ok(value) => ReturnNum {
            value: value as u8,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnNum {
            value: 0, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub extern "C" fn ergo_wallet_address_delete(address: AddressPtr) {
    address_delete(address)
}

// -------------------------------------------------------------------------------------------------
// BlockHeader functions ---------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_block_header_from_json(
    json_str: *const c_char,
    block_header_out: *mut BlockHeaderPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = block_header_from_json(&json, block_header_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_block_header_delete(header: BlockHeaderPtr) {
    block_header_delete(header)
}

// -------------------------------------------------------------------------------------------------
// BlockHeaders functions --------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

make_collection!(BlockHeaders, BlockHeader);

// -------------------------------------------------------------------------------------------------
// PreHeader functions -----------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_preheader_from_block_header(
    block_header: ConstBlockHeaderPtr,
    preheader_out: *mut PreHeaderPtr,
) -> ErrorPtr {
    let res = preheader_from_block_header(block_header, preheader_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_preheader_delete(header: PreHeaderPtr) {
    preheader_delete(header)
}

// -------------------------------------------------------------------------------------------------
// ErgoStateContext functions ----------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_state_context_new(
    pre_header_ptr: ConstPreHeaderPtr,
    headers: ConstBlockHeadersPtr,
    ergo_state_context_out: *mut ErgoStateContextPtr,
) -> ErrorPtr {
    let res = ergo_state_context_new(pre_header_ptr, headers, ergo_state_context_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_state_context_delete(
    ergo_state_context: ErgoStateContextPtr,
) {
    ergo_state_context_delete(ergo_state_context)
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

pub trait IntegerType {}

impl IntegerType for u8 {}
impl IntegerType for usize {}
