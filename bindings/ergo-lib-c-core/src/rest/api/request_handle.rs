use futures_util::future::AbortHandle;

use crate::util::const_ptr_as_ref;
use crate::Error;

pub struct RequestHandle {
    pub abort_handle: AbortHandle,
}

pub type RequestHandlePtr = *mut RequestHandle;

impl From<AbortHandle> for RequestHandle {
    fn from(abort_handle: AbortHandle) -> Self {
        Self { abort_handle }
    }
}

pub unsafe fn request_handle_abort(request_handle: RequestHandlePtr) -> Result<(), Error> {
    let handle = const_ptr_as_ref(request_handle, "request_handle")?;
    handle.abort_handle.abort();
    Ok(())
}
