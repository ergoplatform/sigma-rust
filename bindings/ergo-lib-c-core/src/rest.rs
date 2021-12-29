use std::ffi::c_void;
use std::ptr::NonNull;

use tokio::runtime::Runtime;

use crate::util::const_ptr_as_ref;
use crate::util::mut_ptr_as_mut;
use crate::Error;

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

pub unsafe fn rest_api_node_get_info_async(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletedCallback<NodeInfo>,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    runtime.0.spawn(async move {
        match ergo_lib::ergo_rest::api::get_info(node_conf).await {
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

pub unsafe fn rest_api_node_get_info(
    node_conf_ptr: NodeConfPtr,
    node_info_out: *mut NodeInfoPtr,
) -> Result<(), Error> {
    let runtime_inner = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| Error::Misc(format!("failed to create tokio runtime: {:?}", e).into()))?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let node_info = runtime_inner.block_on(async move {
        ergo_lib::ergo_rest::api::get_info(node_conf)
            .await
            .map_err(|e| Error::Misc(format!("{:?}", e).into()))
    })?;
    *node_info_out = Box::into_raw(Box::new(node_info.into()));
    Ok(())
}
