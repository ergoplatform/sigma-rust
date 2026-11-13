//! DataInput type
use ergo_lib_c_core::{
    data_input::*,
    ergo_box::{BoxIdPtr, ConstBoxIdPtr},
    Error,
};

use crate::delete_ptr;

/// Parse box id (32 byte digest)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_data_input_new(
    box_id_ptr: ConstBoxIdPtr,
    data_input_out: *mut DataInputPtr,
) {
    #[allow(clippy::unwrap_used)]
    data_input_new(box_id_ptr, data_input_out).unwrap();
}

/// Get box id
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_data_input_box_id(
    data_input_ptr: ConstDataInputPtr,
    box_id_out: *mut BoxIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    data_input_box_id(data_input_ptr, box_id_out).unwrap();
}

/// Drop `DataInput`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_data_input_delete(ptr: DataInputPtr) {
    delete_ptr(ptr)
}

make_collection! {
    collection: DataInputs,
    collection_ptr: DataInputsPtr,
    const_collection_ptr: ConstDataInputsPtr,
    item: DataInput,
    item_ptr: DataInputPtr,
    const_item_ptr: ConstDataInputPtr,
    new_fn: ergo_lib_data_inputs_new,
    delete_fn: ergo_lib_data_inputs_delete,
    len_fn: ergo_lib_data_inputs_len,
    get_fn: ergo_lib_data_inputs_get,
    add_fn: ergo_lib_data_inputs_add
}
