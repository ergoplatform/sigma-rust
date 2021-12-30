use ergo_lib_c_core::rest::node_info::NodeInfoPtr;

use crate::delete_ptr;

/// Drop `NodeInfo`
#[no_mangle]
pub extern "C" fn ergo_lib_node_info_delete(ptr: NodeInfoPtr) {
    unsafe { delete_ptr(ptr) }
}
