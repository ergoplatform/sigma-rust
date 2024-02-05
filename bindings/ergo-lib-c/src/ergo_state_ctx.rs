/// Blockchain state (last headers, etc.)
use crate::{block_header::ConstBlockHeadersPtr, delete_ptr, ErrorPtr};
use ergo_lib_c_core::{
    ergo_state_ctx::{ergo_state_context_new, ConstErgoStateContextPtr, ErgoStateContextPtr},
    header::ConstPreHeaderPtr,
    parameters::ConstParametersPtr,
    Error,
};
use paste::paste;

/// Create new context from pre-header
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_state_context_new(
    pre_header_ptr: ConstPreHeaderPtr,
    headers: ConstBlockHeadersPtr,
    parameters: ConstParametersPtr,
    ergo_state_context_out: *mut ErgoStateContextPtr,
) -> ErrorPtr {
    let res = ergo_state_context_new(pre_header_ptr, headers, parameters, ergo_state_context_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_ergo_state_context_delete(ptr: ErgoStateContextPtr) {
    delete_ptr(ptr)
}

make_ffi_eq!(ErgoStateContext);
