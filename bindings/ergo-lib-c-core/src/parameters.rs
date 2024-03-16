//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;

/// Blockchain parameters
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Parameters(pub(crate) chain::parameters::Parameters);
pub type ParametersPtr = *mut Parameters;
pub type ConstParametersPtr = *const Parameters;

/// Return default blockchain parameters that were set at genesis
pub unsafe fn parameters_default(parameters_out: *mut ParametersPtr) {
    *parameters_out = Box::into_raw(Box::new(Parameters(
        chain::parameters::Parameters::default(),
    )));
}

pub unsafe fn parameters_delete(parameters: ParametersPtr) {
    if !parameters.is_null() {
        let boxed = Box::from_raw(parameters);
        std::mem::drop(boxed);
    }
}
