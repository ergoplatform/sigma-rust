use ergo_lib_c_core::block_header::ConstBlockIdPtr;
use ergo_lib_c_core::rest::api::callback::CompletionCallback;
use ergo_lib_c_core::rest::api::node::{
    rest_api_node_get_blocks_header_id_proof_for_tx_id, rest_api_node_get_header,
    rest_api_node_get_info, rest_api_node_get_nipopow_proof_by_header_id,
    rest_api_node_peer_discovery,
};
use ergo_lib_c_core::rest::api::request_handle::RequestHandlePtr;
use ergo_lib_c_core::rest::api::runtime::RestApiRuntimePtr;
use ergo_lib_c_core::rest::node_conf::NodeConfPtr;
use ergo_lib_c_core::transaction::ConstTxIdPtr;
use ergo_lib_c_core::Error;
use ergo_lib_c_core::ErrorPtr;
use std::os::raw::c_char;

/// GET on /info endpoint
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_info(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
) -> ErrorPtr {
    let res = rest_api_node_get_info(runtime_ptr, node_conf_ptr, callback, request_handle_out);
    Error::c_api_from(res)
}

/// GET on /blocks/{blockId}/header endpoint
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_header(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
    header_id_ptr: ConstBlockIdPtr,
) -> ErrorPtr {
    let res = rest_api_node_get_header(
        runtime_ptr,
        node_conf_ptr,
        callback,
        request_handle_out,
        header_id_ptr,
    );
    Error::c_api_from(res)
}

/// GET on /blocks/{header_id}/proofFor/{tx_id} to request the merkle proof for a given transaction
/// that belongs to the given header ID.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_blocks_header_id_proof_for_tx_id(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
    header_id_ptr: ConstBlockIdPtr,
    tx_id_ptr: ConstTxIdPtr,
) -> ErrorPtr {
    let res = rest_api_node_get_blocks_header_id_proof_for_tx_id(
        runtime_ptr,
        node_conf_ptr,
        callback,
        request_handle_out,
        header_id_ptr,
        tx_id_ptr,
    );
    Error::c_api_from(res)
}

/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_nipopow_proof_by_header_id(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
    min_chain_length: u32,
    suffix_len: u32,
    header_id_ptr: ConstBlockIdPtr,
) -> ErrorPtr {
    let res = rest_api_node_get_nipopow_proof_by_header_id(
        runtime_ptr,
        node_conf_ptr,
        callback,
        request_handle_out,
        min_chain_length,
        suffix_len,
        header_id_ptr,
    );
    Error::c_api_from(res)
}

/// GET on /peer_discovery endpoint
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_peer_discovery(
    runtime_ptr: RestApiRuntimePtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
    seeds_ptr: *const *const c_char,
    num_seeds: usize,
    max_parallel_requests: u16,
    timeout_sec: u32,
) -> ErrorPtr {
    let res = rest_api_node_peer_discovery(
        runtime_ptr,
        callback,
        request_handle_out,
        seeds_ptr,
        num_seeds,
        max_parallel_requests,
        timeout_sec,
    );
    Error::c_api_from(res)
}
