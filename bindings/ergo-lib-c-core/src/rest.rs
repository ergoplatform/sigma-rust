use std::ffi::c_void;

use tokio::runtime::Runtime;

use crate::util::const_ptr_as_ref;
use crate::util::mut_ptr_as_mut;
use crate::Error;
use crate::ErrorPtr;

pub struct RestApiRuntime(Runtime);
pub type RestApiRuntimePtr = *mut RestApiRuntime;

pub unsafe fn rest_api_runtime_new(runtime_out: *mut RestApiRuntimePtr) -> Result<(), Error> {
    let runtime_out = mut_ptr_as_mut(runtime_out, "rest_api_runtime_out")?;
    let runtime_inner = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Misc(format!("failed to create tokio runtime: {:?}", e).into()))?;
    *runtime_out = Box::into_raw(Box::new(RestApiRuntime(runtime_inner)));
    Ok(())
}

pub struct NodeConf(ergo_lib::ergo_rest::NodeConf);
pub type NodeConfPtr = *mut NodeConf;

#[derive(derive_more::From, derive_more::Into)]
pub struct NodeInfo(ergo_lib::ergo_rest::NodeInfo);
pub type NodeInfoPtr = *mut NodeInfo;

pub unsafe fn rest_api_node_get_info(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletedCallback<NodeInfoPtr>,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    runtime.0.spawn(async move {
        match ergo_lib::ergo_rest::api::get_info(node_conf).await {
            Ok(node_info) => callback.succeeded(Box::into_raw(Box::new(node_info.into()))),
            Err(e) => callback.failed(Error::c_api_from(Err(Error::Misc(
                format!("{:?}", e).into(),
            )))),
        }
    });
    Ok(())
}

#[repr(C)]
pub struct CompletedCallback<T> {
    userdata_success: *mut c_void,
    userdata_fail: *mut c_void,
    callback_success: extern "C" fn(*mut c_void, T),
    callback_fail: extern "C" fn(*mut c_void, ErrorPtr),
}

unsafe impl<T> Send for CompletedCallback<T> {}

impl<T> CompletedCallback<T> {
    pub fn succeeded(self, t: T) {
        (self.callback_success)(self.userdata_success, t);
        std::mem::forget(self)
    }
    pub fn failed(self, error: ErrorPtr) {
        (self.callback_fail)(self.userdata_fail, error);
        std::mem::forget(self)
    }
}

impl<T> Drop for CompletedCallback<T> {
    fn drop(&mut self) {
        panic!("CompletedCallback must have explicit succeeded or failed call")
    }
}
