use ergo_lib_c_core::rest::api::node_async::rest_api_node_get_info_async;
use ergo_lib_c_core::rest::api::node_async::rest_api_runtime_new;
use ergo_lib_c_core::rest::api::node_async::CompletedCallback;
use ergo_lib_c_core::rest::api::node_async::RestApiRuntimePtr;
use ergo_lib_c_core::rest::node_conf::NodeConfPtr;
use ergo_lib_c_core::rest::node_info::NodeInfo;
use ergo_lib_c_core::Error;
use ergo_lib_c_core::ErrorPtr;

use crate::delete_ptr;

/// Create tokio runtime instance
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_runtime_create(
    runtime_out: *mut RestApiRuntimePtr,
) -> ErrorPtr {
    let res = rest_api_runtime_new(runtime_out);
    Error::c_api_from(res)
}

/// Drop tokio runtime
#[no_mangle]
pub extern "C" fn ergo_lib_rest_api_runtime_delete(ptr: RestApiRuntimePtr) {
    unsafe { delete_ptr(ptr) }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_info_async(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletedCallback<NodeInfo>,
) -> ErrorPtr {
    let res = rest_api_node_get_info_async(runtime_ptr, node_conf_ptr, callback);
    Error::c_api_from(res)
}
