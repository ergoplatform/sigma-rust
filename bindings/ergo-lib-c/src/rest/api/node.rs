use ergo_lib_c_core::rest::api::callback::CompletionCallback;
use ergo_lib_c_core::rest::api::node::rest_api_node_get_info;
use ergo_lib_c_core::rest::api::request_handle::RequestHandlePtr;
use ergo_lib_c_core::rest::api::runtime::RestApiRuntimePtr;
use ergo_lib_c_core::rest::node_conf::NodeConfPtr;
use ergo_lib_c_core::Error;
use ergo_lib_c_core::ErrorPtr;

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
