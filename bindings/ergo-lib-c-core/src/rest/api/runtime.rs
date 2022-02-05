use tokio::runtime::Runtime;

use crate::util::mut_ptr_as_mut;
use crate::Error;

pub struct RestApiRuntime(pub(crate) Runtime);
pub type RestApiRuntimePtr = *mut RestApiRuntime;

pub unsafe fn rest_api_runtime_new(runtime_out: *mut RestApiRuntimePtr) -> Result<(), Error> {
    let runtime_out = mut_ptr_as_mut(runtime_out, "rest_api_runtime_out")?;
    let runtime_inner = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Misc(format!("failed to create tokio runtime: {:?}", e).into()))?;
    *runtime_out = Box::into_raw(Box::new(RestApiRuntime(runtime_inner)));
    Ok(())
}
