use futures_util::future::AbortHandle;
use futures_util::future::Abortable;

use crate::rest::api::runtime::RestApiRuntime;
use crate::Error;

/// Wraps a task with future::Abortable, spawns it on the provided runtime and returns task's abort handle
pub(crate) fn spawn_abortable<T: 'static>(
    runtime: &RestApiRuntime,
    task: T,
) -> Result<AbortHandle, Error>
where
    T: futures_util::Future<Output = ()> + Send,
{
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(task, abort_registration);
    runtime.0.spawn(future);
    Ok(abort_handle)
}
