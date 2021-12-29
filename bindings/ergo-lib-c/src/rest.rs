use ergo_lib_c_core::rest::rest_api_node_get_info;
use ergo_lib_c_core::rest::rest_api_runtime_new;
use ergo_lib_c_core::rest::CompletedCallback;
use ergo_lib_c_core::rest::NodeConfPtr;
use ergo_lib_c_core::rest::NodeInfo;
use ergo_lib_c_core::rest::NodeInfoPtr;
use ergo_lib_c_core::rest::RestApiRuntimePtr;
use ergo_lib_c_core::Error;

use crate::delete_ptr;
use crate::ErrorPtr;

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
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_info(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletedCallback<NodeInfo>,
) -> ErrorPtr {
    let res = rest_api_node_get_info(runtime_ptr, node_conf_ptr, callback);
    Error::c_api_from(res)
}

/// Drop `NodeInfo`
#[no_mangle]
pub extern "C" fn ergo_lib_node_info_delete(ptr: NodeInfoPtr) {
    unsafe { delete_ptr(ptr) }
}

/// Drop `NodeConf`
#[no_mangle]
pub extern "C" fn ergo_lib_node_conf_delete(ptr: NodeConfPtr) {
    unsafe { delete_ptr(ptr) }
}
