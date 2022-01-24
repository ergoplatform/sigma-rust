use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

use ergo_lib::ergo_rest::NodeResponse;

use crate::Error;

#[repr(C)]
pub struct CompletionCallback {
    swift_closure: NonNull<c_void>,
    swift_closure_func: extern "C" fn(NonNull<c_void>, *const c_void, *const Error),
    swift_release_closure_func: extern "C" fn(NonNull<c_void>),
}

unsafe impl Send for CompletionCallback {}

impl CompletionCallback {
    pub fn succeeded<T: NodeResponse>(self, t: T) {
        let ptr = Box::into_raw(Box::new(t)) as *mut _ as *mut c_void;
        (self.swift_closure_func)(self.swift_closure, ptr, ptr::null());
        std::mem::forget(self)
    }
    pub fn failed(self, error: Error) {
        let ptr = Error::c_api_from(Err(error));
        (self.swift_closure_func)(self.swift_closure, ptr::null(), ptr);
        std::mem::forget(self)
    }
}

impl Drop for CompletionCallback {
    fn drop(&mut self) {
        // We only should get here on AbortHandle::abort() call
        // see mem::forget above on callbacks
        // panic!("CompletedCallback must have explicit succeeded or failed call")
    }
}

#[repr(C)]
pub struct ReleaseCallbackWrapper {
    pub swift_closure: NonNull<c_void>,
    pub swift_release_closure_func: extern "C" fn(NonNull<c_void>),
}

impl From<&CompletionCallback> for ReleaseCallbackWrapper {
    fn from(cc: &CompletionCallback) -> Self {
        ReleaseCallbackWrapper {
            swift_closure: cc.swift_closure,
            swift_release_closure_func: cc.swift_release_closure_func,
        }
    }
}
