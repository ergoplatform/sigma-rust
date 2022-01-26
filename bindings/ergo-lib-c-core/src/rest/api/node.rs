//! Async REST API for Ergo node

use crate::rest::node_conf::NodeConfPtr;
use crate::util::const_ptr_as_ref;
use crate::Error;

use self::abortable::spawn_abortable;

use super::callback::CompletionCallback;
use super::callback::ReleaseCallbackWrapper;
use super::request_handle::RequestHandle;
use super::request_handle::RequestHandlePtr;
use super::runtime::RestApiRuntimePtr;

mod abortable;

pub unsafe fn rest_api_node_get_info(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let release_callback: ReleaseCallbackWrapper = (&callback).into();
    let abort_handle = spawn_abortable(runtime, async move {
        match ergo_lib::ergo_rest::api::node::get_info(node_conf).await {
            Ok(node_info) => callback.succeeded(node_info),
            Err(e) => callback.failed(e.into()),
        }
    })?;
    let request_handle = RequestHandle::new(abort_handle, release_callback);
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}
