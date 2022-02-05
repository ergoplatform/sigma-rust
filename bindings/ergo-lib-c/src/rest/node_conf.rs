use std::ffi::CStr;
use std::os::raw::c_char;

use ergo_lib_c_core::rest::node_conf::node_conf_from_addr;
use ergo_lib_c_core::rest::node_conf::NodeConfPtr;
use ergo_lib_c_core::Error;
use ergo_lib_c_core::ErrorPtr;

use crate::delete_ptr;

/// Drop `NodeConf`
#[no_mangle]
pub extern "C" fn ergo_lib_node_conf_delete(ptr: NodeConfPtr) {
    unsafe { delete_ptr(ptr) }
}

/// Parse IP address and port from string
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_node_conf_from_addr(
    addr_str: *const c_char,
    ptr_out: *mut NodeConfPtr,
) -> ErrorPtr {
    let addr = CStr::from_ptr(addr_str).to_string_lossy();
    let res = node_conf_from_addr(&addr, ptr_out);
    Error::c_api_from(res)
}
