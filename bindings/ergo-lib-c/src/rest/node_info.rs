use std::ffi::CString;
use std::os::raw::c_char;

use ergo_lib_c_core::rest::node_info::NodeInfoPtr;
use ergo_lib_c_core::rest::node_info::{node_info_get_name, node_info_is_at_least_version_4_0_100};

use crate::delete_ptr;

/// Drop `NodeInfo`
#[no_mangle]
pub extern "C" fn ergo_lib_node_info_delete(ptr: NodeInfoPtr) {
    unsafe { delete_ptr(ptr) }
}

/// Node's name
#[allow(clippy::unwrap_used)]
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_node_info_get_name(
    ptr: NodeInfoPtr,
    name_str: *mut *const c_char,
) {
    let s = node_info_get_name(ptr);
    *name_str = CString::new(s).unwrap().into_raw();
}

/// Returns true iff the ergo node is at least v4.0.28. This is important since nipopow proofs only
/// work correctly from this version onwards.
#[allow(clippy::unwrap_used)]
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_node_info_is_at_least_version_4_0_100(
    node_info_ptr: NodeInfoPtr,
) -> bool {
    #[allow(clippy::unwrap_used)]
    node_info_is_at_least_version_4_0_100(node_info_ptr).unwrap()
}
