use std::sync::Arc;

use futures_util::future::AbortHandle;

use crate::util::const_ptr_as_ref;
use crate::Error;

use super::callback::{AbortCallback, CallbackGate};

/// A "receipt" of the spawned task
pub struct RequestHandle {
    /// A handle to abort this task
    abort_handle: AbortHandle,
    /// Caller-owned callback invoked only when abort wins the terminal race
    abort_callback: AbortCallback,
    /// Shared terminal gate for the completion/abort callback context
    callback_gate: Arc<CallbackGate>,
}

impl RequestHandle {
    /// Construct a request handle from its legacy abort callback.
    pub fn new(abort_handle: AbortHandle, abort_callback: AbortCallback) -> Self {
        Self::with_gate(abort_handle, abort_callback, CallbackGate::new())
    }

    pub(crate) fn with_gate(
        abort_handle: AbortHandle,
        abort_callback: AbortCallback,
        callback_gate: Arc<CallbackGate>,
    ) -> Self {
        Self {
            abort_handle,
            abort_callback,
            callback_gate,
        }
    }

    /// Aborts the task and calls abort callback
    pub fn abort(&self) {
        self.callback_gate
            .abort_with(&self.abort_callback, || self.abort_handle.abort());
    }
}

pub type RequestHandlePtr = *mut RequestHandle;

pub unsafe fn request_handle_abort(request_handle: RequestHandlePtr) -> Result<(), Error> {
    let handle = const_ptr_as_ref(request_handle, "request_handle")?;
    (*handle).abort();
    Ok(())
}
