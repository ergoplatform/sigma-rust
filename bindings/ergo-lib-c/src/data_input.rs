//! DataInput type
use ergo_lib_c_core::{
    data_input::*,
    ergo_box::{BoxIdPtr, ConstBoxIdPtr},
    Error,
};

use crate::{delete_ptr, ErrorPtr};
use paste::paste;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_data_input_new(
    box_id_ptr: ConstBoxIdPtr,
    data_input_out: *mut DataInputPtr,
) {
    data_input_new(box_id_ptr, data_input_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_data_input_box_id(
    data_input_ptr: ConstDataInputPtr,
    box_id_out: *mut BoxIdPtr,
) {
    data_input_box_id(data_input_ptr, box_id_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_data_input_delete(ptr: DataInputPtr) {
    unsafe { delete_ptr(ptr) }
}

make_collection!(DataInputs, DataInput);
