use ergo_lib_c_core::rest::node_conf::NodeConfPtr;
use ergo_lib_c_core::rest::node_info::NodeInfoPtr;

use crate::delete_ptr;

pub mod api;
pub mod node_conf;
pub mod node_info;

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
