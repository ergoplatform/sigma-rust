use derive_more::{AsRef, From, Into};
use ergo_lib::chain::ergo_state_context::ErgoStateContext as ErgoStateContextInner;
use pyo3::prelude::*;

use super::{
    header::{Header, PreHeader},
    parameters::Parameters,
};

#[pyclass(eq, from_py_object)]
#[derive(Clone, PartialEq, Eq, From, Into, AsRef)]
pub(crate) struct ErgoStateContext(pub(crate) ErgoStateContextInner);

#[pymethods]
impl ErgoStateContext {
    #[new]
    fn new(pre_header: PreHeader, headers: [Header; 10], parameters: Parameters) -> Self {
        Self(ErgoStateContextInner::new(
            pre_header.into(),
            headers.map(Into::into),
            parameters.into(),
        ))
    }
}
