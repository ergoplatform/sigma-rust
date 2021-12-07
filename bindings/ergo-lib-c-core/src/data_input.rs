//! DataInput type

use ergo_lib::chain;

use crate::{
    ergo_box::{BoxId, BoxIdPtr, ConstBoxIdPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Inputs, that are used to enrich script context, but won't be spent by the transaction
#[derive(PartialEq, Debug, Clone)]
pub struct DataInput(pub chain::transaction::DataInput);
pub type DataInputPtr = *mut DataInput;
pub type ConstDataInputPtr = *const DataInput;

/// Parse box id (32 byte digest)
pub unsafe fn data_input_new(
    box_id_ptr: ConstBoxIdPtr,
    data_input_out: *mut DataInputPtr,
) -> Result<(), Error> {
    let box_id = const_ptr_as_ref(box_id_ptr, "box_id_ptr")?;
    let data_input_out = mut_ptr_as_mut(data_input_out, "data_input_out")?;
    *data_input_out = Box::into_raw(Box::new(DataInput(chain::transaction::DataInput {
        box_id: box_id.0.clone(),
    })));
    Ok(())
}

/// Get box id
pub unsafe fn data_input_box_id(
    data_input_ptr: ConstDataInputPtr,
    box_id_out: *mut BoxIdPtr,
) -> Result<(), Error> {
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    let data_input = const_ptr_as_ref(data_input_ptr, "data_input_ptr")?;
    let box_id = BoxId(data_input.0.box_id.clone());
    *box_id_out = Box::into_raw(Box::new(box_id));
    Ok(())
}
