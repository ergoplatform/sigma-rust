//! Unsigned transaction builder

use ergo_lib_c_core::{
    address::{AddressPtr, ConstAddressPtr},
    box_selector::{BoxSelectionPtr, ConstBoxSelectionPtr},
    collections::{CollectionPtr, ConstCollectionPtr},
    data_input::DataInput,
    ergo_box::{BoxValuePtr, ConstBoxValuePtr, ErgoBoxCandidate},
    transaction::UnsignedTransactionPtr,
    tx_builder::*,
    Error, ErrorPtr,
};

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_suggested_tx_fee(value_out: *mut BoxValuePtr) {
    #[allow(clippy::unwrap_used)]
    tx_builder_suggested_tx_fee(value_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_new(
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

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_set_data_inputs(
    tx_builder_mut: TxBuilderPtr,
    data_inputs_ptr: ConstCollectionPtr<DataInput>,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_set_data_inputs(tx_builder_mut, data_inputs_ptr).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_build(
    tx_builder_ptr: ConstTxBuilderPtr,
    unsigned_transaction_out: *mut UnsignedTransactionPtr,
) -> ErrorPtr {
    let res = tx_builder_build(tx_builder_ptr, unsigned_transaction_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_box_selection(
    tx_builder_ptr: ConstTxBuilderPtr,
    box_selection_out: *mut BoxSelectionPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_box_selection(tx_builder_ptr, box_selection_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_data_inputs(
    tx_builder_ptr: ConstTxBuilderPtr,
    data_inputs_out: *mut CollectionPtr<DataInput>,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_data_inputs(tx_builder_ptr, data_inputs_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_output_candidates(
    tx_builder_ptr: ConstTxBuilderPtr,
    output_candidates_out: *mut CollectionPtr<ErgoBoxCandidate>,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_output_candidates(tx_builder_ptr, output_candidates_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_current_height(
    tx_builder_ptr: ConstTxBuilderPtr,
) -> u32 {
    #[allow(clippy::unwrap_used)]
    tx_builder_current_height(tx_builder_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_fee_amount(
    tx_builder_ptr: ConstTxBuilderPtr,
    value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_fee_amount(tx_builder_ptr, value_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_change_address(
    tx_builder_ptr: ConstTxBuilderPtr,
    address_out: *mut AddressPtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_change_address(tx_builder_ptr, address_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_tx_builder_min_change_value(
    tx_builder_ptr: ConstTxBuilderPtr,
    min_change_value_out: *mut BoxValuePtr,
) {
    #[allow(clippy::unwrap_used)]
    tx_builder_min_change_value(tx_builder_ptr, min_change_value_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_tx_builder_delete(ptr: TxBuilderPtr) {
    unsafe { delete_ptr(ptr) }
}
