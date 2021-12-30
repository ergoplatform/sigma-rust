use ergo_lib_c_core::rest::api::node::rest_api_node_get_info;
use ergo_lib_c_core::rest::node_conf::NodeConfPtr;
use ergo_lib_c_core::rest::node_info::NodeInfoPtr;
use ergo_lib_c_core::Error;
use ergo_lib_c_core::ErrorPtr;

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_node_get_info(
    node_conf_ptr: NodeConfPtr,
    node_info_out: *mut NodeInfoPtr,
) -> ErrorPtr {
    let res = rest_api_node_get_info(node_conf_ptr, node_info_out);
    Error::c_api_from(res)
}
