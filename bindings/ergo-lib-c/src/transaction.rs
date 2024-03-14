//! Ergo transaction

use ergo_lib_c_core::{
    collections::{CollectionPtr, ConstCollectionPtr},
    data_input::DataInput,
    ergo_box::{ErgoBox, ErgoBoxCandidate},
    ergo_state_ctx::ConstErgoStateContextPtr,
    input::{Input, UnsignedInput},
    reduced::ConstPropositionsPtr,
    transaction::*,
    util::ByteArray,
    Error, ErrorPtr,
};

use paste::paste;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{delete_ptr, ReturnOption};

// Need to define these here because the generated code from the `make_collection!` macro
// invocations don't yet exist.
type ConstByteArraysPtr = ConstCollectionPtr<ByteArray>;
type DataInputsPtr = CollectionPtr<DataInput>;
type InputsPtr = CollectionPtr<Input>;
type UnsignedInputsPtr = CollectionPtr<UnsignedInput>;
type ErgoBoxCandidatesPtr = CollectionPtr<ErgoBoxCandidate>;
type ErgoBoxesPtr = CollectionPtr<ErgoBox>;

// `CommitmentHint` bindings -----------------------------------------------------------------------

/// Drop `CommitmentHint`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_commitment_hint_delete(ptr: CommitmentHintPtr) {
    delete_ptr(ptr)
}

// `HintsBag` bindings -----------------------------------------------------------------------------

/// Empty HintsBag
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_hints_bag_empty(hints_bag_out: *mut HintsBagPtr) {
    #[allow(clippy::unwrap_used)]
    hints_bag_empty(hints_bag_out).unwrap();
}

/// Add commitment hint to the bag
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_hints_bag_add_commitment(
    hints_bag_mut: HintsBagPtr,
    hint_ptr: ConstCommitmentHintPtr,
) {
    #[allow(clippy::unwrap_used)]
    hints_bag_add_commitment(hints_bag_mut, hint_ptr).unwrap();
}

/// Length of HintsBag
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_hints_bag_len(hints_bag_ptr: ConstHintsBagPtr) -> usize {
    #[allow(clippy::unwrap_used)]
    hints_bag_len(hints_bag_ptr).unwrap()
}

/// Get commitment
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_hints_bag_get(
    hints_bag_ptr: ConstHintsBagPtr,
    index: usize,
    hint_out: *mut CommitmentHintPtr,
) -> ReturnOption {
    match hints_bag_get(hints_bag_ptr, index, hint_out) {
        Ok(is_some) => ReturnOption {
            is_some,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnOption {
            is_some: false, // Dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

/// Drop `HintsBag`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_hints_bag_delete(ptr: HintsBagPtr) {
    delete_ptr(ptr)
}

// `TransactionHintsBag` bindings ------------------------------------------------------------------

/// Empty TransactionHintsBag
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_transaction_hints_bag_empty(
    transaction_hints_bag_out: *mut TransactionHintsBagPtr,
) {
    #[allow(clippy::unwrap_used)]
    transaction_hints_bag_empty(transaction_hints_bag_out).unwrap();
}

/// Adding hints for input
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_transaction_hints_bag_add_hints_for_input(
    transaction_hints_bag_mut: TransactionHintsBagPtr,
    index: usize,
    hints_bag_ptr: ConstHintsBagPtr,
) {
    #[allow(clippy::unwrap_used)]
    transaction_hints_bag_add_hints_for_input(transaction_hints_bag_mut, index, hints_bag_ptr)
        .unwrap();
}

/// Get HintsBag corresponding to input index
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_transaction_hints_bag_all_hints_for_input(
    transaction_hints_bag_ptr: ConstTransactionHintsBagPtr,
    index: usize,
    hints_bag_out: *mut HintsBagPtr,
) {
    #[allow(clippy::unwrap_used)]
    transaction_hints_bag_all_hints_for_input(transaction_hints_bag_ptr, index, hints_bag_out)
        .unwrap();
}

/// Extract hints from signed transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_transaction_extract_hints(
    signed_transaction_ptr: ConstTransactionPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    real_propositions_ptr: ConstPropositionsPtr,
    simulated_propositions_ptr: ConstPropositionsPtr,
    transaction_hints_bag_out: *mut TransactionHintsBagPtr,
) -> ErrorPtr {
    let res = transaction_extract_hints(
        signed_transaction_ptr,
        state_context_ptr,
        boxes_to_spend_ptr,
        data_boxes_ptr,
        real_propositions_ptr,
        simulated_propositions_ptr,
        transaction_hints_bag_out,
    );
    Error::c_api_from(res)
}

/// Drop `TransactionHintsBag`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_transaction_hints_bag_delete(ptr: TransactionHintsBagPtr) {
    delete_ptr(ptr)
}

// `UnsignedTransaction` bindings ------------------------------------------------------------------

/// Get id for transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    tx_id_out: *mut TxIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    unsigned_tx_id(unsigned_tx_ptr, tx_id_out).unwrap();
}

/// Inputs for transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_inputs(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    unsigned_inputs_out: *mut UnsignedInputsPtr,
) {
    #[allow(clippy::unwrap_used)]
    unsigned_tx_inputs(unsigned_tx_ptr, unsigned_inputs_out).unwrap();
}

/// Data inputs for transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_data_inputs(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    data_inputs_out: *mut DataInputsPtr,
) {
    #[allow(clippy::unwrap_used)]
    unsigned_tx_data_inputs(unsigned_tx_ptr, data_inputs_out).unwrap();
}

/// Output candidates for transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_output_candidates(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    ergo_box_candidates_out: *mut ErgoBoxCandidatesPtr,
) {
    #[allow(clippy::unwrap_used)]
    unsigned_tx_output_candidates(unsigned_tx_ptr, ergo_box_candidates_out).unwrap();
}

/// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
/// strings
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_from_json(
    json_str: *const c_char,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = unsigned_tx_from_json(&json, unsigned_tx_out);
    Error::c_api_from(res)
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_to_json(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match unsigned_tx_to_json(unsigned_tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_to_json_eip12(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match unsigned_tx_to_json_eip12(unsigned_tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// Drop `UnsignedTransaction`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_unsigned_tx_delete(ptr: UnsignedTransactionPtr) {
    delete_ptr(ptr)
}

// `Transaction` bindings --------------------------------------------------------------------------

/// Create Transaction from UnsignedTransaction and an array of proofs in the same order as
/// UnsignedTransaction.inputs with empty proof indicated with empty byte array
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_from_unsigned_tx(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    proofs_ptr: ConstByteArraysPtr,
    tx_out: *mut TransactionPtr,
) -> ErrorPtr {
    let res = tx_from_unsigned_tx(unsigned_tx_ptr, proofs_ptr, tx_out);
    Error::c_api_from(res)
}

/// Get id for transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_id(tx_ptr: ConstTransactionPtr, tx_id_out: *mut TxIdPtr) {
    #[allow(clippy::unwrap_used)]
    tx_id(tx_ptr, tx_id_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_inputs(
    tx_ptr: ConstTransactionPtr,
    inputs_out: *mut InputsPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_inputs(tx_ptr, inputs_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_data_inputs(
    tx_ptr: ConstTransactionPtr,
    data_inputs_out: *mut DataInputsPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_data_inputs(tx_ptr, data_inputs_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_output_candidates(
    tx_ptr: ConstTransactionPtr,
    ergo_box_candidates_out: *mut ErgoBoxCandidatesPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_output_candidates(tx_ptr, ergo_box_candidates_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_outputs(
    tx_ptr: ConstTransactionPtr,
    ergo_box_out: *mut ErgoBoxesPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_outputs(tx_ptr, ergo_box_out).unwrap();
}

/// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
/// strings
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_from_json(
    json_str: *const c_char,
    tx_out: *mut TransactionPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = tx_from_json(&json, tx_out);
    Error::c_api_from(res)
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_to_json(
    tx_ptr: ConstTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match tx_to_json(tx_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_to_json_eip12(
    tx_ptr: ConstTransactionPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
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
pub unsafe extern "C" fn ergo_lib_tx_validate(
    tx_ptr: ConstTransactionPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
) -> ErrorPtr {
    Error::c_api_from(tx_validate(
        tx_ptr,
        state_context_ptr,
        boxes_to_spend_ptr,
        data_boxes_ptr,
    ))
}
/// Drop `Transaction`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_delete(ptr: TransactionPtr) {
    delete_ptr(ptr)
}
// `TxId` bindings ----------------------------------------------------------------------

/// Convert a hex string into a TxId
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_id_from_str(
    str: *const c_char,
    tx_id_out: *mut TxIdPtr,
) -> ErrorPtr {
    let str = CStr::from_ptr(str).to_string_lossy();
    let res = tx_id_from_str(&str, tx_id_out);
    Error::c_api_from(res)
}

/// Get the tx id as bytes
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_id_to_str(
    tx_id_ptr: ConstTxIdPtr,
    _str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match tx_id_to_str(tx_id_ptr) {
        Ok(s) => {
            *_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// Drop `TxId`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_id_delete(ptr: TxIdPtr) {
    delete_ptr(ptr)
}

make_ffi_eq!(TxId);
