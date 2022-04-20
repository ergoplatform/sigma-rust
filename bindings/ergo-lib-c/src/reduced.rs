//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use crate::delete_ptr;
use ergo_lib_c_core::{
    collections::ConstCollectionPtr,
    ergo_box::ErgoBox,
    ergo_state_ctx::ConstErgoStateContextPtr,
    reduced::*,
    transaction::{ConstUnsignedTransactionPtr, UnsignedTransactionPtr},
    Error, ErrorPtr,
};

/// Returns `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_reduced_tx_from_unsigned_tx(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    state_context_ptr: ConstErgoStateContextPtr,
    reduced_tx_out: *mut ReducedTransactionPtr,
) -> ErrorPtr {
    let res = reduced_tx_from_unsigned_tx(
        unsigned_tx_ptr,
        boxes_to_spend_ptr,
        data_boxes_ptr,
        state_context_ptr,
        reduced_tx_out,
    );
    Error::c_api_from(res)
}

/// Returns the unsigned transation
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_reduced_tx_unsigned_tx(
    reduced_tx_ptr: ConstReducedTransactionPtr,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) {
    #[allow(clippy::unwrap_used)]
    reduced_tx_unsigned_tx(reduced_tx_ptr, unsigned_tx_out).unwrap();
}

/// Drop `ReducedTransaction`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_reduced_tx_delete(ptr: ReducedTransactionPtr) {
    delete_ptr(ptr)
}

/// Create empty proposition holder
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_propositions_new(propositions_out: *mut PropositionsPtr) {
    #[allow(clippy::unwrap_used)]
    propositions_new(propositions_out).unwrap();
}

/// Adding new proposition
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_propositions_add_proposition_from_bytes(
    propositions_mut: PropositionsPtr,
    bytes_ptr: *const u8,
    len: usize,
) -> ErrorPtr {
    let res = propositions_add_proposition_from_bytes(propositions_mut, bytes_ptr, len);
    Error::c_api_from(res)
}

/// Drop `Propositions`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_propositions_delete(ptr: PropositionsPtr) {
    delete_ptr(ptr)
}
