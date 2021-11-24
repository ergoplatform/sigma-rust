//! Ergo transaction

use ergo_lib_c_core::{
    collections::{CollectionPtr, ConstCollectionPtr},
    data_input::DataInput,
    ergo_box::{ErgoBox, ErgoBoxCandidate},
    input::{Input, UnsignedInput},
    transaction::*,
    util::ByteArray,
    Error, ErrorPtr,
};

use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::delete_ptr;

// Need to define these here because the generated code from the `make_collection!` macro
// invocations don't yet exist.
type ConstByteArraysPtr = ConstCollectionPtr<ByteArray>;
type DataInputsPtr = CollectionPtr<DataInput>;
type InputsPtr = CollectionPtr<Input>;
type UnsignedInputsPtr = CollectionPtr<UnsignedInput>;
type ErgoBoxCandidatesPtr = CollectionPtr<ErgoBoxCandidate>;
type ErgoBoxesPtr = CollectionPtr<ErgoBox>;

// `UnsignedTransaction` bindings ------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    tx_id_out: *mut TxIdPtr,
) -> ErrorPtr {
    let res = unsigned_tx_id(unsigned_tx_ptr, tx_id_out);
    Error::c_api_from(res)
}

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
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_output_candidates(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    ergo_box_candidates_out: *mut ErgoBoxCandidatesPtr,
) -> ErrorPtr {
    let res = unsigned_tx_output_candidates(unsigned_tx_ptr, ergo_box_candidates_out);
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
pub unsafe extern "C" fn ergo_wallet_unsigned_tx_to_json_eip12(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match unsigned_tx_to_json_eip12(unsigned_tx_ptr) {
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

// `Transaction` bindings --------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_from_unsigned_tx(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    proofs_ptr: ConstByteArraysPtr,
    tx_out: *mut TransactionPtr,
) -> ErrorPtr {
    let res = tx_from_unsigned_tx(unsigned_tx_ptr, proofs_ptr, tx_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_id(
    tx_ptr: ConstTransactionPtr,
    tx_id_out: *mut TxIdPtr,
) -> ErrorPtr {
    let res = tx_id(tx_ptr, tx_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_inputs(
    tx_ptr: ConstTransactionPtr,
    inputs_out: *mut InputsPtr,
) -> ErrorPtr {
    let res = tx_inputs(tx_ptr, inputs_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_data_inputs(
    tx_ptr: ConstTransactionPtr,
    data_inputs_out: *mut DataInputsPtr,
) -> ErrorPtr {
    let res = tx_data_inputs(tx_ptr, data_inputs_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_output_candidates(
    tx_ptr: ConstTransactionPtr,
    ergo_box_candidates_out: *mut ErgoBoxCandidatesPtr,
) -> ErrorPtr {
    let res = tx_output_candidates(tx_ptr, ergo_box_candidates_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_outputs(
    tx_ptr: ConstTransactionPtr,
    ergo_box_out: *mut ErgoBoxesPtr,
) -> ErrorPtr {
    let res = tx_outputs(tx_ptr, ergo_box_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_from_json(
    json_str: *const c_char,
    tx_out: *mut TransactionPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = tx_from_json(&json, tx_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_to_json(
    tx_ptr: ConstTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match tx_to_json(tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_to_json_eip12(
    tx_ptr: ConstTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match tx_to_json_eip12(tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_tx_delete(ptr: TransactionPtr) {
    unsafe { delete_ptr(ptr) }
}
// `TxId` bindings ----------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_id_from_str(
    str: *const c_char,
    tx_id_out: *mut TxIdPtr,
) -> ErrorPtr {
    let str = CStr::from_ptr(str).to_string_lossy();
    let res = tx_id_from_str(&str, tx_id_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_id_to_str(
    tx_id_ptr: ConstTxIdPtr,
    _str: *mut *const c_char,
) -> ErrorPtr {
    let res = match tx_id_to_str(tx_id_ptr) {
        Ok(s) => {
            *_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_tx_id_delete(ptr: TxIdPtr) {
    unsafe { delete_ptr(ptr) }
}
