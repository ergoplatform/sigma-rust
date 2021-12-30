use ergo_lib_c_core::rest::node_conf::NodeConfPtr;

use crate::delete_ptr;

/// Drop `NodeConf`
#[no_mangle]
pub extern "C" fn ergo_lib_node_conf_delete(ptr: NodeConfPtr) {
    unsafe { delete_ptr(ptr) }
}
