use ergo_lib_c_core::parameters::{parameters_default, ParametersPtr};

use crate::delete_ptr;

/// Return default blockchain parameters that were set at genesis
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_parameters_default(parameters_out: *mut ParametersPtr) {
    parameters_default(parameters_out);
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_parameters_delete(parameters: ParametersPtr) {
    delete_ptr(parameters)
}
