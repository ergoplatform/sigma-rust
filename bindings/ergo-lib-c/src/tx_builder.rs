//! Unsigned transaction builder

use ergo_lib_c_core::{
    address::{AddressPtr, ConstAddressPtr},
    box_selector::{BoxSelectionPtr, ConstBoxSelectionPtr},
    collections::{CollectionPtr, ConstCollectionPtr},
    context_extension::ContextExtensionPtr,
    data_input::DataInput,
    ergo_box::{BoxIdPtr, BoxValuePtr, ConstBoxValuePtr, ErgoBoxCandidate},
    transaction::UnsignedTransactionPtr,
    tx_builder::*,
    Error, ErrorPtr,
};

use crate::delete_ptr;

/// Suggested transaction fee (semi-default value used across wallets and dApps as of Oct 2020)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_suggested_tx_fee(value_out: *mut BoxValuePtr) {
    #[allow(clippy::unwrap_used)]
    tx_builder_suggested_tx_fee(value_out).unwrap();
}

/// Creates new TxBuilder
/// `box_selection` - selected input boxes (via [`super::box_selector`])
/// `output_candidates` - output boxes to be "created" in this transaction,
/// `current_height` - chain height that will be used in additionally created boxes (change, miner's fee, etc.),
/// `fee_amount` - miner's fee,
/// `change_address` - change (inputs - outputs) will be sent to this address,
/// `min_change_value` - minimal value of the change to be sent to `change_address`, value less than that
/// will be given to miners,
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_new(
    box_selection_ptr: ConstBoxSelectionPtr,
    output_candidates_ptr: ConstCollectionPtr<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount_ptr: ConstBoxValuePtr,
    change_address_ptr: ConstAddressPtr,
    min_change_value_ptr: ConstBoxValuePtr,
    tx_builder_out: *mut TxBuilderPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_new(
        box_selection_ptr,
        output_candidates_ptr,
        current_height,
        fee_amount_ptr,
        change_address_ptr,
        min_change_value_ptr,
        tx_builder_out,
    )
    .unwrap();
}

/// Set transaction's data inputs
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_set_data_inputs(
    tx_builder_mut: TxBuilderPtr,
    data_inputs_ptr: ConstCollectionPtr<DataInput>,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_set_data_inputs(tx_builder_mut, data_inputs_ptr).unwrap();
}

/// Set context extension for a given input
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_set_context_extension(
    tx_builder_mut: TxBuilderPtr,
    box_id_ptr: BoxIdPtr,
    ctx_ext_ptr: ContextExtensionPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_set_context_extension(tx_builder_mut, box_id_ptr, ctx_ext_ptr).unwrap();
}

/// Build the unsigned transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_build(
    tx_builder_ptr: ConstTxBuilderPtr,
    unsigned_transaction_out: *mut UnsignedTransactionPtr,
) -> ErrorPtr {
    let res = tx_builder_build(tx_builder_ptr, unsigned_transaction_out);
    Error::c_api_from(res)
}

/// Get box selection
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_box_selection(
    tx_builder_ptr: ConstTxBuilderPtr,
    box_selection_out: *mut BoxSelectionPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_box_selection(tx_builder_ptr, box_selection_out).unwrap();
}

/// Get data inputs
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_data_inputs(
    tx_builder_ptr: ConstTxBuilderPtr,
    data_inputs_out: *mut CollectionPtr<DataInput>,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_data_inputs(tx_builder_ptr, data_inputs_out).unwrap();
}

/// Get outputs EXCLUDING fee and change
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_output_candidates(
    tx_builder_ptr: ConstTxBuilderPtr,
    output_candidates_out: *mut CollectionPtr<ErgoBoxCandidate>,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_output_candidates(tx_builder_ptr, output_candidates_out).unwrap();
}

/// Get current height
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_current_height(
    tx_builder_ptr: ConstTxBuilderPtr,
) -> u32 {
    #[allow(clippy::unwrap_used)]
    tx_builder_current_height(tx_builder_ptr).unwrap()
}

/// Get fee amount
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_fee_amount(
    tx_builder_ptr: ConstTxBuilderPtr,
    value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_fee_amount(tx_builder_ptr, value_out).unwrap();
}

/// Get change address
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_change_address(
    tx_builder_ptr: ConstTxBuilderPtr,
    address_out: *mut AddressPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_change_address(tx_builder_ptr, address_out).unwrap();
}

/// Get min change value
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_tx_builder_min_change_value(
    tx_builder_ptr: ConstTxBuilderPtr,
    min_change_value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_min_change_value(tx_builder_ptr, min_change_value_out).unwrap();
}

/// Drop `TxBuilder`
#[no_mangle]
pub extern "C" fn ergo_lib_tx_builder_delete(ptr: TxBuilderPtr) {
    unsafe { delete_ptr(ptr) }
}
