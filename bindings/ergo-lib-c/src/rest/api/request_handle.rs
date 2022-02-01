use ergo_lib_c_core::rest::api::request_handle::request_handle_abort;
use ergo_lib_c_core::rest::api::request_handle::RequestHandlePtr;
use ergo_lib_c_core::Error;

use crate::delete_ptr;
use crate::ErrorPtr;

/// Abort the request
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_rest_api_request_handle_abort(
    handle_ptr: RequestHandlePtr,
) -> ErrorPtr {
    let res = request_handle_abort(handle_ptr);
    Error::c_api_from(res)
}

/// Drop request handle
#[no_mangle]
pub extern "C" fn ergo_lib_rest_api_request_handle_delete(ptr: RequestHandlePtr) {
    unsafe { delete_ptr(ptr) }
}
