use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
        #[allow(clippy::forget_non_drop)]
        std::mem::forget(self)
    }

    /// Should be called if task fails (exactly once, thus takes ownership)
    pub fn failed(self, error: Error) {
        let ptr = Error::c_api_from(Err(error));
        (self.completion_callback)(self.user_data, ptr::null(), ptr);
        // free without running the destructor
        #[allow(clippy::forget_non_drop)]
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

/// Shared one-shot terminal gate for a completion callback and its abort callback.
pub(crate) struct CallbackGate {
    claimed: AtomicBool,
}

impl CallbackGate {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            claimed: AtomicBool::new(false),
        })
    }

    fn claim(&self) -> bool {
        self.claimed
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Establish abort as terminal before cancelling the task and releasing caller context.
    pub(crate) fn abort_with(&self, callback: &AbortCallback, abort_task: impl FnOnce()) {
        if self.claim() {
            abort_task();
            callback.abort_callback();
        }
    }
}

/// Completion callback paired with the same terminal gate held by the request handle.
pub(crate) struct GatedCompletionCallback {
    callback: CompletionCallback,
    gate: Arc<CallbackGate>,
}

impl GatedCompletionCallback {
    pub(crate) fn new(callback: CompletionCallback) -> Self {
        let gate = CallbackGate::new();
        Self { callback, gate }
    }

    pub(crate) fn gate(&self) -> Arc<CallbackGate> {
        Arc::clone(&self.gate)
    }

    pub(crate) fn succeeded<T: NodeResponse>(self, value: T) {
        if self.gate.claim() {
            self.callback.succeeded(value);
        }
    }

    pub(crate) fn failed(self, error: Error) {
        if self.gate.claim() {
            self.callback.failed(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::ptr::{self, NonNull};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};

    use futures_util::future::AbortHandle;

    use super::{CompletionCallback, GatedCompletionCallback};
    use crate::rest::api::node::rest_api_node_get_info;
    use crate::rest::api::request_handle::RequestHandle;
    use crate::Error;

    #[derive(Default)]
    struct CallbackEvents {
        completions: AtomicUsize,
        aborts: AtomicUsize,
    }

    extern "C" fn completion_callback(
        user_data: NonNull<c_void>,
        _response: *const c_void,
        error: *const Error,
    ) {
        let events = unsafe { user_data.cast::<CallbackEvents>().as_ref() };
        events.completions.fetch_add(1, Ordering::SeqCst);
        if !error.is_null() {
            unsafe { drop(Box::from_raw(error.cast_mut())) };
        }
    }

    extern "C" fn abort_callback(user_data: NonNull<c_void>) {
        let events = unsafe { user_data.cast::<CallbackEvents>().as_ref() };
        events.aborts.fetch_add(1, Ordering::SeqCst);
    }

    fn callback_fixture() -> (CompletionCallback, NonNull<CallbackEvents>) {
        let events = NonNull::from(Box::leak(Box::new(CallbackEvents::default())));
        let callback = CompletionCallback {
            user_data: events.cast(),
            completion_callback,
            abort_callback,
        };
        (callback, events)
    }

    fn gated_request(callback: CompletionCallback) -> (GatedCompletionCallback, RequestHandle) {
        let abort_callback = (&callback).into();
        let callback = GatedCompletionCallback::new(callback);
        let gate = callback.gate();
        let (abort_handle, _abort_registration) = AbortHandle::new_pair();
        let request = RequestHandle::with_gate(abort_handle, abort_callback, gate);
        (callback, request)
    }

    fn assert_events(events: NonNull<CallbackEvents>, completions: usize, aborts: usize) {
        let events_ref = unsafe { events.as_ref() };
        assert_eq!(events_ref.completions.load(Ordering::SeqCst), completions);
        assert_eq!(events_ref.aborts.load(Ordering::SeqCst), aborts);
        unsafe { drop(Box::from_raw(events.as_ptr())) };
    }

    fn fail(callback: GatedCompletionCallback) {
        callback.failed(Error::InvalidArgument("callback gate test"));
    }

    #[test]
    fn completion_then_abort_has_one_terminal_callback() {
        let (callback, events) = callback_fixture();
        let (callback, request) = gated_request(callback);

        fail(callback);
        request.abort();
        drop(request);

        assert_events(events, 1, 0);
    }

    #[test]
    fn abort_then_completion_has_one_terminal_callback() {
        let (callback, events) = callback_fixture();
        let (callback, request) = gated_request(callback);

        request.abort();
        fail(callback);
        drop(request);

        assert_events(events, 0, 1);
    }

    #[test]
    fn repeated_abort_is_a_no_op_after_the_first_abort() {
        let (callback, events) = callback_fixture();
        let (callback, request) = gated_request(callback);

        request.abort();
        request.abort();
        drop(callback);
        drop(request);

        assert_events(events, 0, 1);
    }

    #[test]
    fn completion_abort_race_has_exactly_one_terminal_callback() {
        for _ in 0..128 {
            let (callback, events) = callback_fixture();
            let (callback, request) = gated_request(callback);
            let barrier = Arc::new(Barrier::new(2));

            let completion_barrier = Arc::clone(&barrier);
            let completion = std::thread::spawn(move || {
                completion_barrier.wait();
                fail(callback);
            });

            barrier.wait();
            request.abort();
            completion.join().expect("completion thread must not panic");
            drop(request);

            let events_ref = unsafe { events.as_ref() };
            assert_eq!(
                events_ref.completions.load(Ordering::SeqCst)
                    + events_ref.aborts.load(Ordering::SeqCst),
                1
            );
            unsafe { drop(Box::from_raw(events.as_ptr())) };
        }
    }

    #[test]
    fn dropping_pending_callback_preserves_existing_no_callback_semantics() {
        let (callback, events) = callback_fixture();
        let (callback, request) = gated_request(callback);

        drop(callback);
        drop(request);

        assert_events(events, 0, 0);
    }

    #[test]
    fn synchronous_validation_error_leaves_callback_with_caller() {
        let (callback, events) = callback_fixture();

        let result = unsafe {
            rest_api_node_get_info(ptr::null_mut(), ptr::null_mut(), callback, ptr::null_mut())
        };

        assert!(result.is_err());
        assert_events(events, 0, 0);
    }
}
