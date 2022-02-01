use futures_util::future::AbortHandle;

use crate::util::const_ptr_as_ref;
use crate::Error;

use super::callback::AbortCallback;

/// A "receipt" of the spawned task
pub struct RequestHandle {
    /// A handle to abort this task
    abort_handle: AbortHandle,
    /// A callback which is called on abort action
    abort_callback: AbortCallback,
}

impl RequestHandle {
    pub fn new(abort_handle: AbortHandle, release_callback: AbortCallback) -> Self {
        Self {
            abort_handle,
            abort_callback: release_callback,
        }
    }

    /// Aborts the task and calls abort callback
    pub fn abort(&self) {
        self.abort_handle.abort();
        self.abort_callback.abort_callback();
    }
}

pub type RequestHandlePtr = *mut RequestHandle;

pub unsafe fn request_handle_abort(request_handle: RequestHandlePtr) -> Result<(), Error> {
    let handle = const_ptr_as_ref(request_handle, "request_handle")?;
    (*handle).abort();
    Ok(())
}
