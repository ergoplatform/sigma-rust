//! Async REST API for Ergo node

use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

use futures_util::future::AbortHandle;
use futures_util::future::Abortable;

use crate::rest::node_conf::NodeConfPtr;
use crate::rest::node_info::NodeInfo;
use crate::util::const_ptr_as_ref;
use crate::Error;

use super::request_handle::RequestHandle;
use super::request_handle::RequestHandlePtr;
use super::runtime::RestApiRuntimePtr;

pub unsafe fn rest_api_node_get_info_async(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletedCallback,
    request_handle_out: *mut RequestHandlePtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;

    let release_callback = ReleaseCallback {
        userdata: callback.userdata,
        callback: callback.callback_release,
    };

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(
        async move {
            match ergo_lib::ergo_rest::api::node::get_info(node_conf).await {
                Ok(node_info) => callback.succeeded(
                    Box::into_raw(Box::new(NodeInfo(node_info))) as *mut _ as *mut c_void
                ),
                Err(e) => callback.failed(Error::c_api_from(Err(Error::Misc(
                    format!("{:?}", e).into(),
                )))),
            }
        },
        abort_registration,
    );
    // abort_handle.abort();

    runtime.0.spawn(future);

    let request_handle = RequestHandle {
        abort_handle,
        release_callback,
    };
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}

#[repr(C)]
pub struct CompletedCallback {
    userdata: NonNull<c_void>,
    callback: extern "C" fn(NonNull<c_void>, *const c_void, *const Error),
    callback_release: extern "C" fn(NonNull<c_void>),
}

unsafe impl Send for CompletedCallback {}

impl CompletedCallback {
    pub fn succeeded(self, t: *const c_void) {
        // TODO: take ownership and wrap into raw pointer here
        (self.callback)(self.userdata, t, ptr::null());
        std::mem::forget(self)
    }
    pub fn failed(self, error: *const Error) {
        // TODO: take ownership and wrap into raw pointer here
        (self.callback)(self.userdata, ptr::null(), error);
        std::mem::forget(self)
    }
}

impl Drop for CompletedCallback {
    fn drop(&mut self) {
        // We only should get here on AbortHandle::abort() call
        // see mem::forget above on callbacks
        // panic!("CompletedCallback must have explicit succeeded or failed call")
    }
}

#[repr(C)]
pub struct ReleaseCallback {
    pub userdata: NonNull<c_void>,
    pub callback: extern "C" fn(NonNull<c_void>),
}
