//! Async REST API for Ergo node

use futures_util::future::AbortHandle;
use futures_util::future::Abortable;

use crate::rest::node_conf::NodeConfPtr;
use crate::rest::node_info::NodeInfo;
use crate::util::const_ptr_as_ref;
use crate::Error;

use super::callback::CompletionCallback;
use super::callback::ReleaseCallbackWrapper;
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

    let release_callback: ReleaseCallbackWrapper = (&callback).into();

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
