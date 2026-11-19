//! Block header

use ergo_lib_c_core::{
    block_header::{
        block_header_from_json, block_header_id, block_header_transactions_root, block_id_from_str,
        BlockHeader, BlockHeaderPtr, BlockId, BlockIdPtr, ConstBlockHeaderPtr, ConstBlockIdPtr,
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

/// Copy the contents of `transactions_root` field to `output`. Key assumption: exactly 32 bytes of
/// memory have been allocated at the address pointed-to by `output`.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_header_transactions_root(
    block_header_ptr: ConstBlockHeaderPtr,
    output: *mut u8,
) -> ErrorPtr {
    let res = block_header_transactions_root(block_header_ptr, output);
    Error::c_api_from(res)
}

/// Delete `BlockHeader`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_header_delete(ptr: BlockHeaderPtr) {
    delete_ptr(ptr)
}

/// Convert a hex string into a BlockId
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_id_from_str(
    str: *const c_char,
    tx_id_out: *mut BlockIdPtr,
) -> ErrorPtr {
    let str = CStr::from_ptr(str).to_string_lossy();
    let res = block_id_from_str(&str, tx_id_out);
    Error::c_api_from(res)
}

/// Delete `BlockId`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_block_id_delete(ptr: BlockIdPtr) {
    delete_ptr(ptr)
}

make_collection! {
    collection: BlockHeaders,
    collection_ptr: BlockHeadersPtr,
    const_collection_ptr: ConstBlockHeadersPtr,
    item: BlockHeader,
    item_ptr: BlockHeaderPtr,
    const_item_ptr: ConstBlockHeaderPtr,
    new_fn: ergo_lib_block_headers_new,
    delete_fn: ergo_lib_block_headers_delete,
    len_fn: ergo_lib_block_headers_len,
    get_fn: ergo_lib_block_headers_get,
    add_fn: ergo_lib_block_headers_add
}
make_ffi_eq! {
    eq_fn: ergo_lib_block_header_eq,
    const_ptr: ConstBlockHeaderPtr
}
make_collection! {
    collection: BlockIds,
    collection_ptr: BlockIdsPtr,
    const_collection_ptr: ConstBlockIdsPtr,
    item: BlockId,
    item_ptr: BlockIdPtr,
    const_item_ptr: ConstBlockIdPtr,
    new_fn: ergo_lib_block_ids_new,
    delete_fn: ergo_lib_block_ids_delete,
    len_fn: ergo_lib_block_ids_len,
    get_fn: ergo_lib_block_ids_get,
    add_fn: ergo_lib_block_ids_add
}
make_ffi_eq! {
    eq_fn: ergo_lib_block_id_eq,
    const_ptr: ConstBlockIdPtr
}
