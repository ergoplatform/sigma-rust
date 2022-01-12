//! Async REST API for Ergo node

use std::ffi::c_void;
use std::ptr::NonNull;

use crate::rest::node_conf::NodeConfPtr;
use crate::rest::node_info::NodeInfo;
use crate::util::const_ptr_as_ref;
use crate::Error;

use super::runtime::RestApiRuntimePtr;

pub unsafe fn rest_api_node_get_info_async(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletedCallback<NodeInfo>,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    runtime.0.spawn(async move {
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
    });
    Ok(())
}

#[repr(C)]
pub struct CompletedCallback<T> {
    userdata_success: NonNull<c_void>,
    userdata_fail: NonNull<c_void>,
    callback_success: extern "C" fn(NonNull<c_void>, NonNull<T>),
    callback_fail: extern "C" fn(NonNull<c_void>, NonNull<Error>),
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
        panic!("CompletedCallback must have explicit succeeded or failed call")
    }
}
