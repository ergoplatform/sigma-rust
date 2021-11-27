//! Block header with the current `spendingTransaction`, that can be predicted by a miner before it's formation
use ergo_lib_c_core::{
    block_header::ConstBlockHeaderPtr,
    header::{preheader_delete, preheader_from_block_header, ConstPreHeaderPtr, PreHeaderPtr},
};
use paste::paste;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_preheader_from_block_header(
    block_header: ConstBlockHeaderPtr,
    preheader_out: *mut PreHeaderPtr,
) {
    #[allow(clippy::unwrap_used)]
    preheader_from_block_header(block_header, preheader_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_preheader_delete(header: PreHeaderPtr) {
    preheader_delete(header)
}

make_ffi_eq!(PreHeader);
