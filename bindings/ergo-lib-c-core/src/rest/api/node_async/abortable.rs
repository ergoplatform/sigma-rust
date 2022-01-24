use futures_util::future::AbortHandle;
use futures_util::future::Abortable;

use crate::rest::api::callback::ReleaseCallbackWrapper;
use crate::rest::api::request_handle::RequestHandle;
use crate::rest::api::runtime::RestApiRuntimePtr;
use crate::util::const_ptr_as_ref;
use crate::Error;

pub(crate) unsafe fn spawn_abortable<T: 'static>(
    runtime_ptr: RestApiRuntimePtr,
    release_callback: ReleaseCallbackWrapper,
    task: T,
) -> Result<RequestHandle, Error>
where
    T: futures_util::Future<Output = ()> + Send,
{
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(task, abort_registration);
    runtime.0.spawn(future);
    let request_handle = RequestHandle {
        abort_handle,
        release_callback,
    };
    Ok(request_handle)
}
