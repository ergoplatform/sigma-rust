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
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;

    let release_callback = (&callback).into();

    // TODO: extract as wrapping func
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(
        async move {
            match ergo_lib::ergo_rest::api::node::get_info(node_conf).await {
                Ok(node_info) => callback.succeeded(NodeInfo(node_info)),
                Err(e) => callback.failed(e.into()),
            }
        },
        abort_registration,
    );

    runtime.0.spawn(future);

    // TODO: make more succint
    let request_handle = RequestHandle {
        abort_handle,
        release_callback,
    };
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}

// TODO: extract
#[repr(C)]
pub struct CompletionCallback {
    swift_closure: NonNull<c_void>,
    swift_closure_func: extern "C" fn(NonNull<c_void>, *const c_void, *const Error),
    swift_release_closure_func: extern "C" fn(NonNull<c_void>),
}

unsafe impl Send for CompletionCallback {}

impl CompletionCallback {
    // TODO: constrain T to avoid passing wrong type errors
    pub fn succeeded<T>(self, t: T) {
        let ptr = Box::into_raw(Box::new(t)) as *mut _ as *mut c_void;
        (self.swift_closure_func)(self.swift_closure, ptr, ptr::null());
        std::mem::forget(self)
    }
    pub fn failed(self, error: Error) {
        let ptr = Error::c_api_from(Err(error));
        (self.swift_closure_func)(self.swift_closure, ptr::null(), ptr);
        std::mem::forget(self)
    }
}

impl Drop for CompletionCallback {
    fn drop(&mut self) {
        // We only should get here on AbortHandle::abort() call
        // see mem::forget above on callbacks
        // panic!("CompletedCallback must have explicit succeeded or failed call")
    }
}

// TODO: extract
#[repr(C)]
pub struct ReleaseCallbackWrapper {
    pub swift_closure: NonNull<c_void>,
    pub swift_release_closure_func: extern "C" fn(NonNull<c_void>),
}

impl From<&CompletionCallback> for ReleaseCallbackWrapper {
    fn from(cc: &CompletionCallback) -> Self {
        ReleaseCallbackWrapper {
            swift_closure: cc.swift_closure,
            swift_release_closure_func: cc.swift_release_closure_func,
        }
    }
}
