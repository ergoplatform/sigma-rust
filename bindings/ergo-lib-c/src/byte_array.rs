//! Byte arrays for use in proofs

use ergo_lib_c_core::{
    util::{byte_array_from_raw_parts, ByteArray, ByteArrayPtr, ConstByteArrayPtr},
    Error, ErrorPtr,
};
use paste::paste;

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_byte_array_from_raw_parts(
    ptr: *const u8,
    len: usize,
    byte_array_out: *mut ByteArrayPtr,
) -> ErrorPtr {
    let res = byte_array_from_raw_parts(ptr, len, byte_array_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_byte_array_delete(ptr: ByteArrayPtr) {
    delete_ptr(ptr)
}

make_collection!(ByteArrays, ByteArray);
