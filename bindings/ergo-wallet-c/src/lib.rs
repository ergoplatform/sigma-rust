//! WASM bindings for sigma-tree

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]

use sigma_tree::chain;

use std::os::raw::{c_int, c_longlong};

/// test
#[no_mangle]
pub extern "C" fn add_numbers(x: c_int, y: c_int) -> c_longlong {
    x as i64 + y as i64
}

// TODO: setup Xcode project and the build pipeline on CI
// TODO: share code with future JNI bindings

// TODO Workflow:
// - parse boxes from JSON and return bytes
// - make a tx (return bytes)
// - convert tx (bytes) to JSON

pub type ErgoStateContextPtr = *mut ergo_wallet::ErgoStateContext;
// TODO wrap enum(TBD) into struct
pub type AddressPtr = *mut chain::Address;
pub type SecretKeyPtr = *mut ergo_wallet::SecretKey;

// TODO: find a way to pass list of boxes?
// TODO: build outputs myself (no sane way to build them on the other side)
// TODO: make return error type

#[no_mangle]
pub extern "C" fn new_signed_transaction(
    _state_context: ErgoStateContextPtr,
    // _inputs: TxInputsPtr,
    // _data_inputs: TxDataInputs,
    // _outputs: TxOutputs,
    _send_change_to: AddressPtr,
    _sk: SecretKeyPtr,
    transaction_out: *mut *const u8,
    len_out: *mut usize,
) -> ErrorPtr {
    todo!()
    // let r = wallet_vote_cast(wallet, settings, proposal, choice, transaction_out, len_out);
    // r.into_c_api()
}
