//! Async REST API for Ergo node

use std::ffi::c_void;
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
    callback: CompletedCallback<NodeInfo>,
    request_handle_out: *mut RequestHandlePtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;

    let release_callback = ReleaseCallback {
        userdata: callback.userdata_success,
        callback: callback.callback_release,
    };

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(
        async move {
            match ergo_lib::ergo_rest::api::node::get_info(node_conf).await {
                Ok(node_info) => callback
                    .succeeded(NonNull::new(Box::into_raw(Box::new(NodeInfo(node_info)))).unwrap()),
                Err(e) => callback.failed(
                    NonNull::new(Error::c_api_from(Err(Error::Misc(
                        format!("{:?}", e).into(),
                    ))))
                    .unwrap(),
                ),
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
pub struct CompletedCallback<T> {
    userdata_success: NonNull<c_void>,
    userdata_fail: NonNull<c_void>,
    callback_success: extern "C" fn(NonNull<c_void>, NonNull<T>),
    callback_fail: extern "C" fn(NonNull<c_void>, NonNull<Error>),
    callback_release: extern "C" fn(NonNull<c_void>),
}

unsafe impl<T> Send for CompletedCallback<T> {}

impl<T> CompletedCallback<T> {
    pub fn succeeded(self, t: NonNull<T>) {
        (self.callback_success)(self.userdata_success, t);
        std::mem::forget(self)
    }
    pub fn failed(self, error: NonNull<Error>) {
        (self.callback_fail)(self.userdata_fail, error);
        std::mem::forget(self)
    }
}

impl<T> Drop for CompletedCallback<T> {
    fn drop(&mut self) {
        // We only should get here on AbortHandle::abort() call
        // see mem::forget above on callbacks

        // TODO: callback closures are leaking here on abort. RequestHandle need to take care of this
        panic!("CompletedCallback must have explicit succeeded or failed call")
    }
}

#[repr(C)]
pub struct ReleaseCallback {
    pub userdata: NonNull<c_void>,
    pub callback: extern "C" fn(NonNull<c_void>),
}
