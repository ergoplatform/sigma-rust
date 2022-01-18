use futures_util::future::AbortHandle;

use crate::util::const_ptr_as_ref;
use crate::Error;

use super::node_async::ReleaseCallback;

pub struct RequestHandle {
    pub abort_handle: AbortHandle,
    pub release_callback: ReleaseCallback,
}

pub type RequestHandlePtr = *mut RequestHandle;

pub unsafe fn request_handle_abort(request_handle: RequestHandlePtr) -> Result<(), Error> {
    let handle = const_ptr_as_ref(request_handle, "request_handle")?;
    handle.abort_handle.abort();
    (handle.release_callback.callback)(handle.release_callback.userdata);
    Ok(())
}
