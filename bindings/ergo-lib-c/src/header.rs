//! Block header with the current `spendingTransaction`, that can be predicted by a miner before it's formation
use ergo_lib_c_core::{
    block_header::ConstBlockHeaderPtr,
    header::{preheader_delete, preheader_from_block_header, PreHeaderPtr},
    Error,
};

use crate::ErrorPtr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_preheader_from_block_header(
    block_header: ConstBlockHeaderPtr,
    preheader_out: *mut PreHeaderPtr,
) -> ErrorPtr {
    let res = preheader_from_block_header(block_header, preheader_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_preheader_delete(header: PreHeaderPtr) {
    preheader_delete(header)
}
