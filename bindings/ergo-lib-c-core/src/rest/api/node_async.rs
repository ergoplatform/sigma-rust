//! Async REST API for Ergo node

use crate::rest::node_conf::NodeConfPtr;
use crate::rest::node_info::NodeInfo;
use crate::util::const_ptr_as_ref;
use crate::Error;

use self::abortable::spawn_abortable;

use super::callback::CompletionCallback;
use super::callback::ReleaseCallbackWrapper;
use super::request_handle::RequestHandlePtr;
use super::runtime::RestApiRuntimePtr;

mod abortable;

pub unsafe fn rest_api_node_get_info_async(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let release_callback: ReleaseCallbackWrapper = (&callback).into();
    let request_handle = spawn_abortable(runtime, release_callback, async move {
        match ergo_lib::ergo_rest::api::node::get_info(node_conf).await {
            Ok(node_info) => callback.succeeded(NodeInfo(node_info)),
            Err(e) => callback.failed(e.into()),
        }
    })?;
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}
