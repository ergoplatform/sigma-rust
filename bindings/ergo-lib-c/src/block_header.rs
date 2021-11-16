//! Block header
use paste::paste;

use ergo_lib_c_core::{
    block_header::{
        block_header_delete, block_header_from_json, BlockHeader, BlockHeaderPtr,
        ConstBlockHeaderPtr,
    },
    Error,
};
use std::{ffi::CStr, os::raw::c_char};

use crate::ErrorPtr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_block_header_from_json(
    json_str: *const c_char,
    block_header_out: *mut BlockHeaderPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = block_header_from_json(&json, block_header_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_block_header_delete(header: BlockHeaderPtr) {
    block_header_delete(header)
}

// -------------------------------------------------------------------------------------------------
// BlockHeaders functions --------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

make_collection!(BlockHeaders, BlockHeader);
