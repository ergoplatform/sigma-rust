//! Block header
use paste::paste;

use ergo_lib_c_core::{
    block_header::{
        block_header_from_json, block_header_id, BlockHeader, BlockHeaderPtr, BlockIdPtr,
        ConstBlockHeaderPtr,
    },
    Error,
};
use std::{ffi::CStr, os::raw::c_char};

use crate::{delete_ptr, ErrorPtr};

/// Parse BlockHeader array from JSON (Node API)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_header_from_json(
    json_str: *const c_char,
    block_header_out: *mut BlockHeaderPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = block_header_from_json(&json, block_header_out);
    Error::c_api_from(res)
}

/// Get `BlockHeader`s id
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_header_id(
    block_header_ptr: ConstBlockHeaderPtr,
    block_id_out: *mut BlockIdPtr,
) {
    #[allow(clippy::unwrap_used)]
    block_header_id(block_header_ptr, block_id_out).unwrap();
}

/// Delete `BlockHeader`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_header_delete(ptr: BlockHeaderPtr) {
    delete_ptr(ptr)
}

/// Delete `BlockId`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_id_delete(ptr: BlockIdPtr) {
    delete_ptr(ptr)
}

make_collection!(BlockHeaders, BlockHeader);
make_ffi_eq!(BlockHeader);
