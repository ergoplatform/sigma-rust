//! Unsigned transaction builder
use ergo_lib::wallet;

use crate::{
    address::{Address, AddressPtr, ConstAddressPtr},
    box_selector::{BoxSelection, BoxSelectionPtr, ConstBoxSelectionPtr},
    collections::{Collection, CollectionPtr, ConstCollectionPtr},
    data_input::DataInput,
    ergo_box::{BoxValue, BoxValuePtr, ConstBoxValuePtr, ErgoBoxCandidate},
    transaction::{UnsignedTransaction, UnsignedTransactionPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Unsigned transaction builder
pub struct TxBuilder(
    wallet::tx_builder::TxBuilder<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox>,
);
pub type TxBuilderPtr = *mut TxBuilder;
pub type ConstTxBuilderPtr = *const TxBuilder;

pub unsafe fn tx_builder_suggested_tx_fee(value_out: *mut BoxValuePtr) -> Result<(), Error> {
    let value_out = mut_ptr_as_mut(value_out, "value_out")?;
    *value_out = Box::into_raw(Box::new(BoxValue(wallet::tx_builder::SUGGESTED_TX_FEE())));
    Ok(())
}

pub unsafe fn tx_builder_new(
    box_selection_ptr: ConstBoxSelectionPtr,
    output_candidates_ptr: ConstCollectionPtr<ErgoBoxCandidate>,
    current_height: u32,
    fee_amount_ptr: ConstBoxValuePtr,
    change_address_ptr: ConstAddressPtr,
    min_change_value_ptr: ConstBoxValuePtr,
    tx_builder_out: *mut TxBuilderPtr,
) -> Result<(), Error> {
    let box_selection = const_ptr_as_ref(box_selection_ptr, "box_selection_ptr")?;
    let output_candidates = const_ptr_as_ref(output_candidates_ptr, "output_candidates_ptr")?;
    let fee_amount = const_ptr_as_ref(fee_amount_ptr, "fee_amount_ptr")?;
    let change_address = const_ptr_as_ref(change_address_ptr, "change_address_ptr")?;
    let min_change_value = const_ptr_as_ref(min_change_value_ptr, "min_change_value_ptr")?;
    let tx_builder_out = mut_ptr_as_mut(tx_builder_out, "tx_builder_out")?;
    *tx_builder_out = Box::into_raw(Box::new(TxBuilder(
        ergo_lib::wallet::tx_builder::TxBuilder::new(
            box_selection.0.clone(),
            output_candidates
                .0
                .clone()
                .into_iter()
                .map(|b| b.0)
                .collect(),
            current_height,
            fee_amount.0,
            change_address.0.clone(),
            min_change_value.0,
        ),
    )));
    Ok(())
}

pub unsafe fn tx_builder_set_data_inputs(
    tx_builder_mut: TxBuilderPtr,
    data_inputs_ptr: ConstCollectionPtr<DataInput>,
) -> Result<(), Error> {
    let data_inputs = const_ptr_as_ref(data_inputs_ptr, "data_inputs_ptr")?;
    let tx_builder_mut = mut_ptr_as_mut(tx_builder_mut, "tx_builder_mut")?;
    tx_builder_mut
        .0
        .set_data_inputs(data_inputs.0.clone().into_iter().map(|d| d.0).collect());
    Ok(())
}

pub unsafe fn tx_builder_build(
    tx_builder_ptr: ConstTxBuilderPtr,
    unsigned_transaction_out: *mut UnsignedTransactionPtr,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let unsigned_transaction_out =
        mut_ptr_as_mut(unsigned_transaction_out, "unsigned_transaction_out")?;
    let unsigned_tx = tx_builder
        .0
        .clone()
        .build()
        .map(UnsignedTransaction)
        .map_err(|_| Error::Misc("TxBuilder.build(): error".into()))?;
    *unsigned_transaction_out = Box::into_raw(Box::new(unsigned_tx));
    Ok(())
}

pub unsafe fn tx_builder_box_selection(
    tx_builder_ptr: ConstTxBuilderPtr,
    box_selection_out: *mut BoxSelectionPtr,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let box_selection_out = mut_ptr_as_mut(box_selection_out, "box_selection_out")?;
    *box_selection_out = Box::into_raw(Box::new(BoxSelection(tx_builder.0.box_selection())));
    Ok(())
}

pub unsafe fn tx_builder_data_inputs(
    tx_builder_ptr: ConstTxBuilderPtr,
    data_inputs_out: *mut CollectionPtr<DataInput>,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let data_inputs_out = mut_ptr_as_mut(data_inputs_out, "data_inputs_out")?;
    *data_inputs_out = Box::into_raw(Box::new(Collection(
        tx_builder
            .0
            .data_inputs()
            .into_iter()
            .map(DataInput)
            .collect(),
    )));
    Ok(())
}

pub unsafe fn tx_builder_output_candidates(
    tx_builder_ptr: ConstTxBuilderPtr,
    output_candidates_out: *mut CollectionPtr<ErgoBoxCandidate>,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let output_candidates_out = mut_ptr_as_mut(output_candidates_out, "output_candidates_out")?;
    *output_candidates_out = Box::into_raw(Box::new(Collection(
        tx_builder
            .0
            .output_candidates()
            .into_iter()
            .map(ErgoBoxCandidate)
            .collect(),
    )));
    Ok(())
}

pub unsafe fn tx_builder_current_height(tx_builder_ptr: ConstTxBuilderPtr) -> Result<u32, Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    Ok(tx_builder.0.current_height())
}

pub unsafe fn tx_builder_fee_amount(
    tx_builder_ptr: ConstTxBuilderPtr,
    value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let value_out = mut_ptr_as_mut(value_out, "value_out")?;
    *value_out = Box::into_raw(Box::new(BoxValue(tx_builder.0.fee_amount())));
    Ok(())
}

pub unsafe fn tx_builder_change_address(
    tx_builder_ptr: ConstTxBuilderPtr,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let address_out = mut_ptr_as_mut(address_out, "address_out")?;
    *address_out = Box::into_raw(Box::new(Address(tx_builder.0.change_address())));
    Ok(())
}

pub unsafe fn tx_builder_min_change_value(
    tx_builder_ptr: ConstTxBuilderPtr,
    min_change_value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let tx_builder = const_ptr_as_ref(tx_builder_ptr, "tx_builder_ptr")?;
    let min_change_value_out = mut_ptr_as_mut(min_change_value_out, "min_change_value_out")?;
    *min_change_value_out = Box::into_raw(Box::new(BoxValue(tx_builder.0.min_change_value())));
    Ok(())
}
