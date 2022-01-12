use ergo_lib_c_core::rest::api::runtime::rest_api_runtime_new;
use ergo_lib_c_core::rest::api::runtime::RestApiRuntimePtr;
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
