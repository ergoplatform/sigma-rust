//! Block header with the current `spendingTransaction`, that can be predicted by a miner before its
//! formation
use ergo_lib_c_core::{
    block_header::ConstBlockHeaderPtr,
    header::{preheader_from_block_header, ConstPreHeaderPtr, PreHeaderPtr},
};
use paste::paste;

use crate::delete_ptr;

/// Create instance using data from block header
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_preheader_from_block_header(
    block_header: ConstBlockHeaderPtr,
    preheader_out: *mut PreHeaderPtr,
) {
    #[allow(clippy::unwrap_used)]
    preheader_from_block_header(block_header, preheader_out).unwrap();
}

/// Drop `PreHeader`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_preheader_delete(ptr: PreHeaderPtr) {
    delete_ptr(ptr)
}

make_ffi_eq!(PreHeader);
