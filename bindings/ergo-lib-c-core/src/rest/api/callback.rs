use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

use ergo_lib::ergo_rest::NodeResponse;

use crate::Error;

/// Callback info for async task
#[repr(C)]
pub struct CompletionCallback {
    /// Caller's data passed back to the user on the callback
    user_data: NonNull<c_void>,
    /// User's completion callback function, where the first arg is the above user_data
    /// following by either response data or an error
    completion_callback: extern "C" fn(NonNull<c_void>, *const c_void, *const Error),
    /// User's abort callback, where the argument is the above user_data
    abort_callback: extern "C" fn(NonNull<c_void>),
}

unsafe impl Send for CompletionCallback {}

impl CompletionCallback {
    /// Should be called on succesfull task execution (exactly once, thus takes ownership)
    pub fn succeeded<T: NodeResponse>(self, t: T) {
        let ptr = Box::into_raw(Box::new(t)) as *mut _ as *mut c_void;
        (self.completion_callback)(self.user_data, ptr, ptr::null());
        // free without running the destructor
        std::mem::forget(self)
    }

    /// Should be called if task fails (exactly once, thus takes ownership)
    pub fn failed(self, error: Error) {
        let ptr = Error::c_api_from(Err(error));
        (self.completion_callback)(self.user_data, ptr::null(), ptr);
        // free without running the destructor
        std::mem::forget(self)
    }
}

/// Abort callback info for async task
#[repr(C)]
pub struct AbortCallback {
    /// Caller's data passed back to the user on the callback
    user_data: NonNull<c_void>,
    /// User's abort callback, where the argument is the above user_data
    abort_callback: extern "C" fn(NonNull<c_void>),
}

impl AbortCallback {
    /// Call the user's abort callback
    pub fn abort_callback(&self) {
        (self.abort_callback)(self.user_data);
    }
}

impl From<&CompletionCallback> for AbortCallback {
    fn from(cc: &CompletionCallback) -> Self {
        AbortCallback {
            user_data: cc.user_data,
            abort_callback: cc.abort_callback,
        }
    }
}
