/// Blockchain state (last headers, etc.)
use crate::{block_header::ConstBlockHeadersPtr, ErrorPtr};
use ergo_lib_c_core::{
    ergo_state_ctx::{ergo_state_context_delete, ergo_state_context_new, ErgoStateContextPtr},
    header::ConstPreHeaderPtr,
    Error,
};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_state_context_new(
    pre_header_ptr: ConstPreHeaderPtr,
    headers: ConstBlockHeadersPtr,
    ergo_state_context_out: *mut ErgoStateContextPtr,
) -> ErrorPtr {
    let res = ergo_state_context_new(pre_header_ptr, headers, ergo_state_context_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_ergo_state_context_delete(
    ergo_state_context: ErgoStateContextPtr,
) {
    ergo_state_context_delete(ergo_state_context)
}
