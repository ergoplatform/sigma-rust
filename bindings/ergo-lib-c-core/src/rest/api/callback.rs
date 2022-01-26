use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

use ergo_lib::ergo_rest::NodeResponse;

use crate::Error;

/// Callback info from the Swift side
#[repr(C)]
pub struct CompletionCallback {
    /// Wrapped Swift closure pointer (see WrapClosure.swift)
    swift_closure: NonNull<c_void>,
    /// The above closure wrapped into a C compatible callback (see WrapClosure.swift)
    swift_closure_func: extern "C" fn(NonNull<c_void>, *const c_void, *const Error),
    /// C compatible closure that just frees(reduces the reference count of) the above closure on the Swift side
    swift_release_closure_func: extern "C" fn(NonNull<c_void>),
}

unsafe impl Send for CompletionCallback {}

impl CompletionCallback {
    /// Should be called on succesfull task execution (exactly once, thus takes ownership)
    pub fn succeeded<T: NodeResponse>(self, t: T) {
        let ptr = Box::into_raw(Box::new(t)) as *mut _ as *mut c_void;
        (self.swift_closure_func)(self.swift_closure, ptr, ptr::null());
        // free without running the destructor
        std::mem::forget(self)
    }

    /// Should be called if task fails (exactly once, thus takes ownership)
    pub fn failed(self, error: Error) {
        let ptr = Error::c_api_from(Err(error));
        (self.swift_closure_func)(self.swift_closure, ptr::null(), ptr);
        // free without running the destructor
        std::mem::forget(self)
    }
}

#[repr(C)]
pub struct ReleaseCallbackWrapper {
    /// Wrapped Swift closure pointer (see WrapClosure.swift)
    swift_closure: NonNull<c_void>,
    /// C compatible closure that just frees(reduces the reference count of) the above closure on the Swift side
    swift_release_closure_func: extern "C" fn(NonNull<c_void>),
}

impl ReleaseCallbackWrapper {
    /// Call the user's closure release func
    pub fn release_callback(&self) {
        (self.swift_release_closure_func)(self.swift_closure);
    }
}

impl From<&CompletionCallback> for ReleaseCallbackWrapper {
    fn from(cc: &CompletionCallback) -> Self {
        ReleaseCallbackWrapper {
            swift_closure: cc.swift_closure,
            swift_release_closure_func: cc.swift_release_closure_func,
        }
    }
}
