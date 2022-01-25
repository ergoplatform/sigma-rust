use futures_util::future::AbortHandle;

use crate::util::const_ptr_as_ref;
use crate::Error;

use super::callback::ReleaseCallbackWrapper;

/// A "receipt" of the spawned task
pub struct RequestHandle {
    /// A handle to abort this task
    abort_handle: AbortHandle,
    /// A callback to release user's closure on the Swift side
    release_callback: ReleaseCallbackWrapper,
}

impl RequestHandle {
    pub fn new(abort_handle: AbortHandle, release_callback: ReleaseCallbackWrapper) -> Self {
        Self {
            abort_handle,
            release_callback,
        }
    }

    /// Aborts the task and calls the release of user's closure on the Swift side
    pub fn abort(&self) {
        self.abort_handle.abort();
        self.release_callback.release_callback();
    }
}

pub type RequestHandlePtr = *mut RequestHandle;

pub unsafe fn request_handle_abort(request_handle: RequestHandlePtr) -> Result<(), Error> {
    let handle = const_ptr_as_ref(request_handle, "request_handle")?;
    (*handle).abort();
    Ok(())
}
