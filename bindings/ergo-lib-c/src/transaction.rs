//! Ergo transaction

use ergo_lib_c_core::{
    collections::CollectionPtr, data_input::DataInput, input::UnsignedInput, transaction::*, Error,
    ErrorPtr,
};

use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    tx_id_ptr: *mut *const c_char,
) -> ErrorPtr {
    let res = match unsigned_tx_id(unsigned_tx_ptr) {
        Ok(s) => {
            *tx_id_ptr = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

// Need to define these here because the generated code from the `make_collection!` macro
// invocations don't yet exist.
type DataInputsPtr = CollectionPtr<DataInput>;
type UnsignedInputsPtr = CollectionPtr<UnsignedInput>;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_inputs(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    unsigned_inputs_out: *mut UnsignedInputsPtr,
) -> ErrorPtr {
    let res = unsigned_tx_inputs(unsigned_tx_ptr, unsigned_inputs_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_data_inputs(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    data_inputs_out: *mut DataInputsPtr,
) -> ErrorPtr {
    let res = unsigned_tx_data_inputs(unsigned_tx_ptr, data_inputs_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_from_json(
    json_str: *const c_char,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = unsigned_tx_from_json(&json, unsigned_tx_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_to_json(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match unsigned_tx_to_json(unsigned_tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_unsigned_tx_delete(ptr: UnsignedTransactionPtr) {
    unsafe { delete_ptr(ptr) }
}
